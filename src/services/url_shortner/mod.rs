use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::models::url::{ActiveModel, Column, Entity as Url};

fn generate_code() -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

pub async fn create_short_url(
    db: &DatabaseConnection,
    long_url: &str,
) -> Result<String, sea_orm::DbErr> {
    let code = loop {
        let candidate = generate_code();
        let exists = Url::find()
            .filter(Column::ShortCode.eq(&candidate))
            .one(db)
            .await?
            .is_some();
        if !exists {
            break candidate;
        }
    };

    let url = ActiveModel {
        short_code: Set(code.clone()),
        long_url: Set(long_url.to_owned()),
        ..Default::default()
    };
    url.insert(db).await?;

    Ok(code)
}

pub async fn resolve_short_url(
    db: &DatabaseConnection,
    code: &str,
) -> Result<Option<String>, sea_orm::DbErr> {
    let result = Url::find()
        .filter(Column::ShortCode.eq(code))
        .one(db)
        .await?;
    Ok(result.map(|m| m.long_url))
}
