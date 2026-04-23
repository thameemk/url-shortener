use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "url_clicks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub url_id: i32,
    pub clicked_at: DateTimeWithTimeZone,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::url::Entity",
        from = "Column::UrlId",
        to = "super::url::Column::Id",
        on_delete = "Cascade"
    )]
    Url,
}

impl Related<super::url::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Url.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
