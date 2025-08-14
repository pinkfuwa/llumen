use base64::{Engine, engine::general_purpose::STANDARD};
use entity::config;
use pasetors::{keys::SymmetricKey, version4::V4};
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn load_key(conn: &DatabaseConnection) -> anyhow::Result<SymmetricKey<V4>> {
    let model = config::Entity::find_by_id("auth_key").one(conn).await?;
    let model = model.ok_or(anyhow::anyhow!("missing auth_key in database"))?;
    let decoded_key = STANDARD.decode(model.value)?;
    Ok(SymmetricKey::from(&decoded_key)?)
}
