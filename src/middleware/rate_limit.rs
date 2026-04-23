use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use governor::{
    clock::{Clock, DefaultClock},
    state::keyed::DashMapStateStore,
    Quota, RateLimiter,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroU32,
    sync::Arc,
};

pub type KeyedRateLimiter =
    Arc<RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock>>;

pub fn new_rate_limiter(per_minute: u32) -> KeyedRateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(per_minute).expect("rate must be > 0"));
    Arc::new(RateLimiter::dashmap(quota))
}

fn client_ip(req: &Request) -> IpAddr {
    // Prefer forwarded headers set by reverse proxies / load balancers.
    if let Some(val) = req.headers().get("x-forwarded-for") {
        if let Ok(s) = val.to_str() {
            if let Some(first) = s.split(',').next() {
                if let Ok(ip) = first.trim().parse() {
                    return ip;
                }
            }
        }
    }
    if let Some(val) = req.headers().get("x-real-ip") {
        if let Ok(s) = val.to_str() {
            if let Ok(ip) = s.trim().parse() {
                return ip;
            }
        }
    }
    req.extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

pub async fn rate_limit_middleware(
    limiter: KeyedRateLimiter,
    req: Request,
    next: Next,
) -> Response {
    let ip = client_ip(&req);
    match limiter.check_key(&ip) {
        Ok(_) => next.run(req).await,
        Err(not_until) => {
            let retry_after = not_until
                .wait_time_from(DefaultClock::default().now())
                .as_secs()
                .max(1);
            let mut res = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": "too many requests",
                    "retry_after_seconds": retry_after
                })),
            )
                .into_response();
            if let Ok(v) = retry_after.to_string().parse() {
                res.headers_mut().insert("retry-after", v);
            }
            res
        }
    }
}
