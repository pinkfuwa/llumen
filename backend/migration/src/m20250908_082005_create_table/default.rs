// password hash of P@88w0rd
const PASSWORD_HASH_ENCODE: &str =
    "$argon2id$v=19$m=16,t=2,p=1$aTg5eTNyMmRzLTA$FM4qzh9B/+DdCVOiQQruGw";

const DEFAULT_MODEL_CONFIG: &str = r#"
display_name="GPT-OSS 20B"
# From https://openrouter.ai/models
# don't put "online" suffix.
model_id="openai/gpt-oss-20b:free"

[capability]
# allow user to upload image, the model need to support it
# set to false to disallow upload despite its support
image = false
audio = false
# available option: Native, Text, Mistral, Disabled
ocr = "Native"
"#;

use pasetors::keys::Generate;
use sea_orm_migration::prelude::*;

use super::entity::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Set default admin user
        let default_admin = Query::insert()
            .into_table(User::Table)
            .columns([User::Name, User::Password])
            .values_panic(["admin".into(), PASSWORD_HASH_ENCODE.into()])
            .to_owned();
        manager.exec_stmt(default_admin).await?;

        // add paseto key
        let key = pasetors::keys::SymmetricKey::generate().expect("Cannot generate");
        let insert_key = Query::insert()
            .into_table(Config::Table)
            .columns([Config::Key, Config::Value])
            .values_panic(["paseto_key".into(), key.as_bytes().into()])
            .to_owned();
        manager.exec_stmt(insert_key).await?;

        // default model
        let default_model = Query::insert()
            .into_table(Model::Table)
            .columns([Model::Config])
            .values_panic([DEFAULT_MODEL_CONFIG.trim().into()])
            .to_owned();
        manager.exec_stmt(default_model).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
