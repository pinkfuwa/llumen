use std::sync::Arc;

use anyhow::{Context as _, Result};
use entity::{ChunkKind, MessageKind, chunk};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

use crate::chat::{CompletionContext, Context, Token};

/// Deep research pipeline that orchestrates multiple agents for comprehensive research
pub struct DeepPipeline {
    ctx: Arc<Context>,
    completion_ctx: CompletionContext,
}

impl DeepPipeline {
    pub fn new(ctx: Arc<Context>, completion_ctx: CompletionContext) -> Self {
        Self {
            ctx,
            completion_ctx,
        }
    }

    pub async fn process(mut self) -> Result<()> {
        // Check if the model supports tool calling
        let model_config = self
            .completion_ctx
            .model
            .get_config()
            .context("corruptted database")?;

        if !model_config.is_tool_capable() {
            return self.completion_ctx.save(Some("Deep research requires a model with tool calling support. Please select a different model.")).await;
        }

        let message_id = self.completion_ctx.get_message_id();

        // For now, send a simple placeholder message
        // TODO: Implement the full deep research flow
        let plan_json = serde_json::json!({
            "steps": [
                {
                    "id": "step1",
                    "description": "Deep research is under development",
                    "need_search": false,
                    "status": "completed"
                }
            ],
            "has_enough_context": false
        });

        // Send plan token
        self.completion_ctx.add_token_force(Token::ResearchPlan(
            serde_json::to_string(&plan_json).unwrap(),
        ));

        // Send a simple report
        self.completion_ctx
            .add_token_force(Token::ResearchReport(
                "Deep research functionality is currently under development. This is a placeholder response.".to_string(),
            ));

        // Store chunks
        let chunks = vec![
            chunk::ActiveModel {
                message_id: Set(message_id),
                kind: Set(ChunkKind::Plan),
                content: Set(serde_json::to_string(&plan_json).unwrap()),
                ..Default::default()
            },
            chunk::ActiveModel {
                message_id: Set(message_id),
                kind: Set(ChunkKind::Report),
                content: Set("Deep research functionality is currently under development. This is a placeholder response.".to_string()),
                ..Default::default()
            },
        ];

        let chunk_results = chunk::Entity::insert_many(chunks)
            .exec(&self.ctx.db)
            .await
            .context("Failed to insert chunks")?;

        // Send completion token
        self.completion_ctx.add_token_force(Token::Complete {
            message_id,
            chunk_ids: vec![
                chunk_results.last_insert_id,
                chunk_results.last_insert_id + 1,
            ],
            cost: 0.0,
            token: 0,
        });

        Ok(())
    }
}
