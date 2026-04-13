use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelIdsReq {}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelIdsResp {
    pub ids: Vec<String>,
}

fn merge_model_ids(mut model_ids: Vec<String>, video_model_ids: Vec<String>) -> Vec<String> {
    model_ids.extend(video_model_ids);
    model_ids.sort();
    model_ids.dedup();
    model_ids
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(_): Json<ModelIdsReq>,
) -> JsonResult<ModelIdsResp> {
    let ids = merge_model_ids(
        app.openrouter.get_model_ids().await,
        app.openrouter.get_video_model_ids().await,
    );
    Ok(Json(ModelIdsResp { ids }))
}

#[cfg(test)]
mod tests {
    use super::merge_model_ids;

    #[test]
    fn merge_model_ids_deduplicates_and_sorts() {
        let ids = merge_model_ids(
            vec!["openai/gpt-4".to_string(), "anthropic/claude".to_string()],
            vec!["anthropic/claude".to_string(), "google/gemini".to_string()],
        );

        assert_eq!(
            ids,
            vec![
                "anthropic/claude".to_string(),
                "google/gemini".to_string(),
                "openai/gpt-4".to_string()
            ]
        );
    }
}
