use std::sync::Arc;

use anyhow::{Context as _, Result};
use entity::{ChunkKind, DeepPlan, DeepReport, DeepStep, DeepStepStatus, chunk};
use sea_orm::{ActiveValue::Set, EntityTrait};

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
            .context("corrupted database")?;

        if !model_config.is_tool_capable() {
            return self.completion_ctx.save(Some("Deep research requires a model with tool calling support. Please select a different model.")).await;
        }

        let message_id = self.completion_ctx.get_message_id();

        // Phase 1: Create and stream research plan
        let plan = self.create_research_plan().await?;
        
        // Stream plan incrementally
        let plan_json = serde_json::to_string(&plan).unwrap();
        self.stream_plan(&plan_json).await?;

        // Phase 2: Execute research steps
        let mut step_results = Vec::new();
        for (idx, step) in plan.steps.iter().enumerate() {
            let result = self.execute_step(step, idx).await?;
            step_results.push(result);
        }

        // Phase 3: Generate and stream final report
        let report = self.generate_report(&plan, &step_results).await?;
        self.stream_report(&report).await?;

        // Store all chunks in database
        let chunks = self.create_chunks(message_id, &plan, &step_results, &report);
        let chunk_results = chunk::Entity::insert_many(chunks)
            .exec(&self.ctx.db)
            .await
            .context("Failed to insert chunks")?;

        // Calculate chunk IDs
        let num_chunks = 1 + step_results.len() + 1; // plan + steps + report
        let chunk_ids: Vec<i32> = (0..num_chunks as i32)
            .map(|i| chunk_results.last_insert_id + i)
            .collect();

        // Send completion token
        self.completion_ctx.add_token_force(Token::Complete {
            message_id,
            chunk_ids,
            cost: 0.0,
            token: 0,
        });

        Ok(())
    }

    async fn create_research_plan(&self) -> Result<DeepPlan> {
        // TODO: Call planner agent with user query
        // For now, create a simple demo plan
        let user_message = self
            .completion_ctx
            .latest_user_message()
            .unwrap_or("research topic");
        
        Ok(DeepPlan {
            steps: vec![
                DeepStep {
                    id: "step1".to_string(),
                    description: format!("Research and gather information about: {}", user_message),
                    need_search: true,
                    status: DeepStepStatus::InProgress,
                    result: None,
                },
                DeepStep {
                    id: "step2".to_string(),
                    description: "Analyze and synthesize gathered information".to_string(),
                    need_search: false,
                    status: DeepStepStatus::InProgress,
                    result: None,
                },
            ],
            has_enough_context: false,
        })
    }

    async fn stream_plan(&mut self, plan_json: &str) -> Result<()> {
        // Stream plan incrementally to frontend
        const CHUNK_SIZE: usize = 50;
        for chunk in plan_json.as_bytes().chunks(CHUNK_SIZE) {
            let chunk_str = String::from_utf8_lossy(chunk).to_string();
            self.completion_ctx
                .add_token_force(Token::ResearchPlan(chunk_str));
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }

    async fn execute_step(&mut self, step: &DeepStep, idx: usize) -> Result<DeepStep> {
        // Stream step as in-progress
        let mut current_step = step.clone();
        current_step.status = DeepStepStatus::InProgress;
        
        let step_json = serde_json::to_string(&current_step).unwrap();
        self.stream_step(&step_json).await?;

        // TODO: Execute actual research step
        // For now, simulate processing
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Mark step as completed with result
        current_step.status = DeepStepStatus::Completed;
        current_step.result = Some(format!(
            "Completed step {}: {}",
            idx + 1,
            step.description
        ));

        // Stream updated step
        let step_json = serde_json::to_string(&current_step).unwrap();
        self.stream_step(&step_json).await?;

        Ok(current_step)
    }

    async fn stream_step(&mut self, step_json: &str) -> Result<()> {
        // Stream step incrementally to frontend
        const CHUNK_SIZE: usize = 50;
        for chunk in step_json.as_bytes().chunks(CHUNK_SIZE) {
            let chunk_str = String::from_utf8_lossy(chunk).to_string();
            self.completion_ctx
                .add_token_force(Token::ResearchStep(chunk_str));
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }

    async fn generate_report(
        &self,
        _plan: &DeepPlan,
        step_results: &[DeepStep],
    ) -> Result<DeepReport> {
        // TODO: Call reporter agent with step results
        // For now, create a simple summary report
        let mut report_content = String::from("# Research Report\n\n");
        report_content.push_str("## Summary\n\n");
        
        for (idx, step) in step_results.iter().enumerate() {
            report_content.push_str(&format!("### Step {}\n\n", idx + 1));
            report_content.push_str(&format!("**Description**: {}\n\n", step.description));
            if let Some(result) = &step.result {
                report_content.push_str(&format!("**Result**: {}\n\n", result));
            }
        }

        report_content.push_str("\n## Conclusion\n\n");
        report_content.push_str("Research completed successfully. This is a demonstration of the deep research pipeline structure.\n");

        Ok(DeepReport {
            content: report_content,
        })
    }

    async fn stream_report(&mut self, report: &DeepReport) -> Result<()> {
        // Stream report incrementally to frontend
        const CHUNK_SIZE: usize = 100;
        for chunk in report.content.as_bytes().chunks(CHUNK_SIZE) {
            let chunk_str = String::from_utf8_lossy(chunk).to_string();
            self.completion_ctx
                .add_token_force(Token::ResearchReport(chunk_str));
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        }
        Ok(())
    }

    fn create_chunks(
        &self,
        message_id: i32,
        plan: &DeepPlan,
        step_results: &[DeepStep],
        report: &DeepReport,
    ) -> Vec<chunk::ActiveModel> {
        let mut chunks = Vec::new();

        // Add plan chunk
        chunks.push(chunk::ActiveModel {
            message_id: Set(message_id),
            kind: Set(ChunkKind::Plan),
            content: Set(serde_json::to_string(plan).unwrap()),
            ..Default::default()
        });

        // Add step chunks
        for step in step_results {
            chunks.push(chunk::ActiveModel {
                message_id: Set(message_id),
                kind: Set(ChunkKind::Step),
                content: Set(serde_json::to_string(step).unwrap()),
                ..Default::default()
            });
        }

        // Add report chunk
        chunks.push(chunk::ActiveModel {
            message_id: Set(message_id),
            kind: Set(ChunkKind::Report),
            content: Set(serde_json::to_string(report).unwrap()),
            ..Default::default()
        });

        chunks
    }
}
