use chrono::{Duration, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::NotSet;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "urls")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub short_code: String,
    pub long_url: String,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub expires_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            if let NotSet = self.expires_at {
                let default = (Utc::now() + Duration::days(10)).into();
                self.expires_at = sea_orm::Set(Some(default));
            }
        }
        Ok(self)
    }
}
