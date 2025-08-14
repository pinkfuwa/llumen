pub use sea_orm_migration::prelude::*;

mod m20250724_144358_create_tables;
mod m20250814_065208_add_default_config;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250724_144358_create_tables::Migration),
            Box::new(m20250814_065208_add_default_config::Migration),
        ]
    }
}
