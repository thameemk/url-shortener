use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UrlClicks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UrlClicks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UrlClicks::UrlId).integer().not_null())
                    .col(
                        ColumnDef::new(UrlClicks::ClickedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(UrlClicks::IpAddress).string())
                    .col(ColumnDef::new(UrlClicks::UserAgent).text())
                    .col(ColumnDef::new(UrlClicks::Referer).text())
                    .foreign_key(
                        ForeignKey::create()
                            .from(UrlClicks::Table, UrlClicks::UrlId)
                            .to(Alias::new("urls"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(UrlClicks::Table)
                    .col(UrlClicks::UrlId)
                    .name("idx_url_clicks_url_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UrlClicks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UrlClicks {
    Table,
    Id,
    UrlId,
    ClickedAt,
    IpAddress,
    UserAgent,
    Referer,
}
