pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_urls_table;
mod m20260421_172413_add_expires_at_to_urls;
mod m20260421_000000_backfill_expires_at;
mod m20260422_000000_create_url_clicks_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_urls_table::Migration),
            Box::new(m20260421_172413_add_expires_at_to_urls::Migration),
            Box::new(m20260421_000000_backfill_expires_at::Migration),
            Box::new(m20260422_000000_create_url_clicks_table::Migration),
        ]
    }
}
