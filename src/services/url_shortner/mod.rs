use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use crate::models::url::{ActiveModel, Column, Entity as Url, Model};

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
) -> Result<Model, sea_orm::DbErr> {
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
        short_code: Set(code),
        long_url: Set(long_url.to_owned()),
        ..Default::default()
    };
    url.insert(db).await
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
) -> Result<Option<Model>, sea_orm::DbErr> {
    let existing = Url::find_by_id(id).one(db).await?;
    match existing {
        None => Ok(None),
        Some(model) => {
            let mut active: ActiveModel = model.into();
            active.long_url = Set(long_url.to_owned());
            let updated = active.update(db).await?;
            Ok(Some(updated))
        }
    }
}
