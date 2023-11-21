#![allow(clippy::too_many_lines)]

pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;

lazy_static::lazy_static! {
    pub static ref CURRENT_TIMESTAMP: SimpleExpr = SimpleExpr::Custom(r"(strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))".to_owned());
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}
