use sea_orm_migration::prelude::*;
use std::{env, path::PathBuf};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // If DATA_PATH is set, construct DATABASE_URL from it
    if let Ok(data_path) = env::var("DATA_PATH") {
        let mut db_path = PathBuf::from(data_path);
        db_path.push("db.sqlite");
        let database_url = format!(
            "sqlite://{}?mode=rwc",
            db_path.display().to_string().replace('\\', "/")
        );
        env::set_var("DATABASE_URL", database_url);
    }

    cli::run_cli(migration::Migrator).await;
}
