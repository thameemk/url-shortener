use chrono::Utc;
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use sea_orm::prelude::DateTimeWithTimeZone;

use crate::models::url::{ActiveModel, Column, Entity as Url, Model};

pub enum ResolveResult {
    Found(String),
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
            Ok(ResolveResult::Found(m.long_url))
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
