use rand::Rng;
use sqlx::PgPool;

fn generate_code() -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

pub async fn create_short_url(db: &PgPool, long_url: &str) -> Result<String, sqlx::Error> {
    let code = loop {
        let candidate = generate_code();
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM urls WHERE short_code = $1)")
                .bind(&candidate)
                .fetch_one(db)
                .await?;
        if !exists {
            break candidate;
        }
    };

    sqlx::query("INSERT INTO urls (short_code, long_url) VALUES ($1, $2)")
        .bind(&code)
        .bind(long_url)
        .execute(db)
        .await?;

    Ok(code)
}

pub async fn resolve_short_url(db: &PgPool, code: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT long_url FROM urls WHERE short_code = $1")
        .bind(code)
        .fetch_optional(db)
        .await
}
