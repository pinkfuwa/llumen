use std::sync::Arc;

use anyhow::{Context as _, Result};
use entity::{ChunkKind, DeepPlan, DeepReport, DeepStep, DeepStepStatus, chunk};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::chat::{CompletionContext, Context, Token, prompt::PromptKind};
use crate::openrouter;

/// Planner response structure matching the prompt output
#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlannerResponse {
    locale: String,
    has_enough_context: bool,
    thought: String,
    title: String,
    steps: Vec<PlannerStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlannerStep {
    need_search: bool,
    title: String,
    description: String,
    step_type: String,
}

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

        // Phase 2: Execute research steps (if needed)
        let mut step_results = Vec::new();
        if !plan.has_enough_context {
            for (idx, step) in plan.steps.iter().enumerate() {
                let result = self.execute_step(step, idx).await?;
                step_results.push(result);
            }
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
        // Render planner prompt
        let system_prompt = self
            .ctx
            .prompt
            .render(PromptKind::DeepPlanner, &self.completion_ctx)
            .context("Failed to render planner prompt")?;

        let user_message = self
            .completion_ctx
            .latest_user_message()
            .unwrap_or("research topic");

        // Call LLM with planner prompt
        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(user_message.to_string()),
        ];

        let model_config = self
            .completion_ctx
            .model
            .get_config()
            .context("Failed to get model config")?;

        let model: openrouter::Model = model_config.into();

        let completion = self
            .ctx
            .openrouter
            .complete(messages, model)
            .await
            .context("Failed to call planner agent")?;

        // Parse planner response
        let planner_response: PlannerResponse = serde_json::from_str(&completion.response)
            .context("Failed to parse planner response")?;

        // Convert to DeepPlan format
        let steps = planner_response
            .steps
            .into_iter()
            .enumerate()
            .map(|(idx, step)| DeepStep {
                id: format!("step{}", idx + 1),
                description: format!("{}: {}", step.title, step.description),
                need_search: step.need_search,
                status: DeepStepStatus::InProgress,
                result: None,
            })
            .collect();

        Ok(DeepPlan {
            steps,
            has_enough_context: planner_response.has_enough_context,
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

        // If step doesn't need search, just mark as completed
        if !step.need_search {
            current_step.status = DeepStepStatus::Completed;
            current_step.result = Some(format!(
                "Processing step completed: {}",
                step.description
            ));
            
            let step_json = serde_json::to_string(&current_step).unwrap();
            self.stream_step(&step_json).await?;
            return Ok(current_step);
        }

        // Execute research step with researcher agent
        match self.execute_research_step(step).await {
            Ok(result) => {
                current_step.status = DeepStepStatus::Completed;
                current_step.result = Some(result);
            }
            Err(e) => {
                log::error!("Research step {} failed: {:?}", idx + 1, e);
                current_step.status = DeepStepStatus::Failed;
                current_step.result = Some(format!("Failed: {}", e));
            }
        }

        // Stream updated step
        let step_json = serde_json::to_string(&current_step).unwrap();
        self.stream_step(&step_json).await?;

        Ok(current_step)
    }

    async fn execute_research_step(&self, step: &DeepStep) -> Result<String> {
        // Render researcher prompt
        let system_prompt = self
            .ctx
            .prompt
            .render(PromptKind::DeepResearcher, &self.completion_ctx)
            .context("Failed to render researcher prompt")?;

        // Build messages
        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(step.description.clone()),
        ];

        let model_config = self
            .completion_ctx
            .model
            .get_config()
            .context("Failed to get model config")?;

        let mut model: openrouter::Model = model_config.into();
        model.online = true; // Enable online mode for web search

        // Stream the completion
        let stream = self
            .ctx
            .openrouter
            .stream(messages, &model, vec![])
            .await
            .context("Failed to start researcher agent stream")?;

        // Collect the research result
        let mut result = String::new();
        let mut stream = Box::pin(stream);
        
        use tokio_stream::StreamExt;
        while let Some(resp) = stream.next().await {
            match resp {
                Ok(openrouter::StreamCompletionResp::ResponseToken(token)) => {
                    result.push_str(&token);
                }
                Ok(_) => {}
                Err(e) => {
                    log::error!("Stream error during research: {:?}", e);
                    break;
                }
            }
        }

        if result.is_empty() {
            result = "No research results obtained.".to_string();
        }

        Ok(result)
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
        // Render reporter prompt
        let system_prompt = self
            .ctx
            .prompt
            .render(PromptKind::DeepReporter, &self.completion_ctx)
            .context("Failed to render reporter prompt")?;

        // Build context from step results
        let mut context = String::from("# Research Results\n\n");
        for (idx, step) in step_results.iter().enumerate() {
            context.push_str(&format!("## Step {}: {}\n\n", idx + 1, step.description));
            
            if let Some(result) = &step.result {
                context.push_str(&format!("{}\n\n", result));
            } else {
                context.push_str("No results available.\n\n");
            }
            
            context.push_str("---\n\n");
        }

        // Call reporter agent
        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(context),
        ];

        let model_config = self
            .completion_ctx
            .model
            .get_config()
            .context("Failed to get model config")?;

        let model: openrouter::Model = model_config.into();

        let completion = self
            .ctx
            .openrouter
            .complete(messages, model)
            .await
            .context("Failed to call reporter agent")?;

        Ok(DeepReport {
            content: completion.response,
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
