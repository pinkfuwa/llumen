pub use sea_orm_migration::prelude::*;

mod m20250724_144358_create_tables;
mod m20250824_044739_add_user_config;
mod m20250828_114136_add_default_model;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250724_144358_create_tables::Migration),
            Box::new(m20250824_044739_add_user_config::Migration),
            Box::new(m20250828_114136_add_default_model::Migration),
        ]
    }
}
