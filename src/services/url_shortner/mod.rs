use chrono::Utc;
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use sea_orm::prelude::DateTimeWithTimeZone;

use crate::models::url::{ActiveModel, Column, Entity as Url, Model};
use crate::models::url_click::{
    ActiveModel as ClickActiveModel, Column as ClickColumn, Entity as UrlClick,
    Model as ClickModel,
};

pub enum ResolveResult {
    Found(String, i32), // (long_url, url_id)
    NotFound,
    Expired,
}

pub enum ShortUrlError {
    CodeTaken,
    Db(sea_orm::DbErr),
}

impl From<sea_orm::DbErr> for ShortUrlError {
    fn from(e: sea_orm::DbErr) -> Self {
        ShortUrlError::Db(e)
    }
}

impl std::fmt::Display for ShortUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortUrlError::CodeTaken => write!(f, "short code is already taken"),
            ShortUrlError::Db(e) => write!(f, "{e}"),
        }
    }
}

fn generate_code() -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

async fn is_code_taken(
    db: &DatabaseConnection,
    code: &str,
    exclude_id: Option<i32>,
) -> Result<bool, sea_orm::DbErr> {
    let mut query = Url::find().filter(Column::ShortCode.eq(code));
    if let Some(id) = exclude_id {
        query = query.filter(Column::Id.ne(id));
    }
    Ok(query.one(db).await?.is_some())
}

pub async fn create_short_url(
    db: &DatabaseConnection,
    long_url: &str,
    short_code: Option<&str>,
    expires_at: Option<DateTimeWithTimeZone>,
) -> Result<Model, ShortUrlError> {
    let code = match short_code {
        Some(code) => {
            if is_code_taken(db, code, None).await? {
                return Err(ShortUrlError::CodeTaken);
            }
            code.to_owned()
        }
        None => loop {
            let candidate = generate_code();
            if !is_code_taken(db, &candidate, None).await? {
                break candidate;
            }
        },
    };

    let url = ActiveModel {
        short_code: Set(code),
        long_url: Set(long_url.to_owned()),
        expires_at: match expires_at {
            Some(v) => Set(Some(v)),
            None => sea_orm::ActiveValue::NotSet,
        },
        ..Default::default()
    };
    Ok(url.insert(db).await?)
}

pub struct ClickRecord {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

pub async fn record_click(
    db: &DatabaseConnection,
    url_id: i32,
    record: ClickRecord,
) -> Result<(), sea_orm::DbErr> {
    ClickActiveModel {
        url_id: Set(url_id),
        ip_address: Set(record.ip_address),
        user_agent: Set(record.user_agent),
        referer: Set(record.referer),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

pub struct UrlAnalytics {
    pub total_clicks: u64,
    pub clicks: Vec<ClickModel>,
}

pub async fn get_url_analytics(
    db: &DatabaseConnection,
    url_id: i32,
    limit: u64,
) -> Result<Option<UrlAnalytics>, sea_orm::DbErr> {
    if Url::find_by_id(url_id).one(db).await?.is_none() {
        return Ok(None);
    }

    let total_clicks = UrlClick::find()
        .filter(ClickColumn::UrlId.eq(url_id))
        .count(db)
        .await?;

    let clicks = UrlClick::find()
        .filter(ClickColumn::UrlId.eq(url_id))
        .order_by_desc(ClickColumn::ClickedAt)
        .limit(limit)
        .all(db)
        .await?;

    Ok(Some(UrlAnalytics {
        total_clicks,
        clicks,
    }))
}

pub async fn resolve_short_url(
    db: &DatabaseConnection,
    code: &str,
) -> Result<ResolveResult, sea_orm::DbErr> {
    let result = Url::find()
        .filter(Column::ShortCode.eq(code))
        .one(db)
        .await?;

    match result {
        None => Ok(ResolveResult::NotFound),
        Some(m) => {
            if let Some(expires_at) = m.expires_at {
                if expires_at.with_timezone(&Utc) <= Utc::now() {
                    return Ok(ResolveResult::Expired);
                }
            }
            Ok(ResolveResult::Found(m.long_url, m.id))
        }
    }
}

pub async fn list_urls(
    db: &DatabaseConnection,
    page: u64,
    per_page: u64,
) -> Result<(Vec<Model>, u64), sea_orm::DbErr> {
    let paginator = Url::find().order_by_asc(Column::Id).paginate(db, per_page);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page.saturating_sub(1)).await?;
    Ok((items, total))
}

pub async fn get_url_by_id(
    db: &DatabaseConnection,
    id: i32,
) -> Result<Option<Model>, sea_orm::DbErr> {
    Url::find_by_id(id).one(db).await
}

pub async fn update_url(
    db: &DatabaseConnection,
    id: i32,
    long_url: &str,
    short_code: Option<&str>,
    expires_at: Option<DateTimeWithTimeZone>,
) -> Result<Option<Model>, ShortUrlError> {
    let existing = Url::find_by_id(id).one(db).await?;
    match existing {
        None => Ok(None),
        Some(model) => {
            let mut active: ActiveModel = model.into();
            active.long_url = Set(long_url.to_owned());
            active.expires_at = Set(expires_at);
            if let Some(code) = short_code {
                if is_code_taken(db, code, Some(id)).await? {
                    return Err(ShortUrlError::CodeTaken);
                }
                active.short_code = Set(code.to_owned());
            }
            let updated = active.update(db).await?;
            Ok(Some(updated))
        }
    }
}
