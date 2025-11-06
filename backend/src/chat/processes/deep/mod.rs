mod helper;
mod prompt;
pub mod tools;

use std::sync::Arc;

use anyhow::{Context as _, Result};
use entity::{ChunkKind, DeepPlan, DeepReport, DeepStep, DeepStepStatus, chunk};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::Deserialize;
use tokio_stream::StreamExt;

use crate::chat::{CompletionContext, Context, Token, context::StreamEndReason};
use crate::openrouter;

use helper::{PlannerResponse, PlannerStep};
use prompt::DeepPrompt;

/// Deep research pipeline that orchestrates multiple agents for comprehensive research
pub struct DeepPipeline {
    ctx: Arc<Context>,
    completion_ctx: CompletionContext,
    prompt: DeepPrompt,
    model: openrouter::Model,
}

impl DeepPipeline {
    pub fn new(ctx: Arc<Context>, completion_ctx: CompletionContext) -> Self {
        let model_config = completion_ctx
            .model
            .get_config()
            .expect("Failed to get model config");
        
        let mut model: openrouter::Model = model_config.into();
        model.online = true; // Enable online mode for web search

        Self {
            ctx,
            completion_ctx,
            prompt: DeepPrompt::new(),
            model,
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
            return self
                .completion_ctx
                .save(Some(
                    "Deep research requires a model with tool calling support. Please select a different model.",
                ))
                .await;
        }

        let message_id = self.completion_ctx.get_message_id();

        // Phase 0: Coordinator (check if we should handoff to planner)
        let should_plan = self.run_coordinator().await?;
        
        if !should_plan {
            // If coordinator didn't handoff to planner, we're done
            return self.completion_ctx.save(None::<String>).await;
        }

        // Optional: Enhance the prompt
        let enhanced_prompt = self.enhance_prompt().await?;

        // Phase 1: Create and stream research plan
        let plan = self.create_research_plan(&enhanced_prompt).await?;

        // Stream plan incrementally
        let plan_json = serde_json::to_string(&plan).unwrap();
        self.stream_plan(&plan_json).await?;

        // Phase 2: Execute research steps (if needed)
        let mut step_results = Vec::new();
        if !plan.has_enough_context {
            let plan_title = enhanced_prompt.clone();
            for (idx, step) in plan.steps.iter().enumerate() {
                let result = self
                    .execute_step(step, idx, &plan_title, &step_results)
                    .await?;
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

    async fn run_coordinator(&mut self) -> Result<bool> {
        let system_prompt = self
            .prompt
            .render_coordinator(&self.completion_ctx)
            .context("Failed to render coordinator prompt")?;

        let user_message = self
            .completion_ctx
            .latest_user_message()
            .unwrap_or("")
            .to_string();

        // Create handoff_to_planner tool
        let handoff_tool = openrouter::Tool {
            name: "handoff_to_planner".to_string(),
            description: "Handoff to planner agent to do plan.".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "research_topic": {
                        "type": "string",
                        "description": "The topic of the research task to be handed off."
                    },
                    "local": {
                        "type": "string",
                        "description": "The user's detected language locale (e.g., en-US, zh-TW)."
                    }
                },
                "required": ["research_topic", "local"]
            }),
        };

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(user_message),
        ];

        let mut res = self
            .ctx
            .openrouter
            .stream(messages, &self.model, vec![handoff_tool])
            .await?;

        let halt = self
            .completion_ctx
            .put_stream((&mut res).map(|resp| resp.map(Into::into)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            log::debug!("The stream was halted");
        }

        let result = res.get_result();

        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        // Check if handoff_to_planner was called
        if let Some(tool_call) = result.toolcall {
            if tool_call.name == "handoff_to_planner" {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn enhance_prompt(&mut self) -> Result<String> {
        let system_prompt = self
            .prompt
            .render_prompt_enhancer(&self.completion_ctx)
            .context("Failed to render prompt enhancer prompt")?;

        let user_message = self
            .completion_ctx
            .latest_user_message()
            .unwrap_or("")
            .to_string();

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(user_message.clone()),
        ];

        let mut res = self
            .ctx
            .openrouter
            .stream(messages, &self.model, vec![])
            .await?;

        let halt = self
            .completion_ctx
            .put_stream((&mut res).map(|resp| resp.map(Into::into)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            log::debug!("The stream was halted");
        }

        let result = res.get_result();

        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        // Extract enhanced prompt from <enhanced_prompt> tags
        let response: String = result
            .responses
            .iter()
            .filter_map(|r| match r {
                openrouter::StreamCompletionResp::ResponseToken(token) => Some(token.as_str()),
                _ => None,
            })
            .collect();

        // Extract content from XML tags
        if let Some(start) = response.find("<enhanced_prompt>") {
            if let Some(end) = response.find("</enhanced_prompt>") {
                let enhanced = &response[start + 17..end];
                return Ok(enhanced.trim().to_string());
            }
        }

        // Fallback to original message if no tags found
        Ok(user_message)
    }

    async fn create_research_plan(&mut self, enhanced_prompt: &str) -> Result<DeepPlan> {
        let system_prompt = self
            .prompt
            .render_planner(&self.completion_ctx)
            .context("Failed to render planner prompt")?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(enhanced_prompt.to_string()),
        ];

        let mut res = self
            .ctx
            .openrouter
            .stream(messages, &self.model, vec![])
            .await?;

        let halt = self
            .completion_ctx
            .put_stream((&mut res).map(|resp| resp.map(Into::into)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            log::debug!("The stream was halted");
        }

        let result = res.get_result();

        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        // Parse planner response
        let response: String = result
            .responses
            .iter()
            .filter_map(|r| match r {
                openrouter::StreamCompletionResp::ResponseToken(token) => Some(token.as_str()),
                _ => None,
            })
            .collect();

        let planner_response: PlannerResponse = serde_json::from_str(&response).context(format!(
            "Failed to parse planner response as JSON. Response: {}",
            response.chars().take(500).collect::<String>()
        ))?;

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

    async fn execute_step(
        &mut self,
        step: &DeepStep,
        idx: usize,
        plan_title: &str,
        completed_steps: &[DeepStep],
    ) -> Result<DeepStep> {
        // Stream step as in-progress
        let mut current_step = step.clone();
        current_step.status = DeepStepStatus::InProgress;

        let step_json = serde_json::to_string(&current_step).unwrap();
        self.stream_step(&step_json).await?;

        // Execute the step based on whether it needs search
        let result = if step.need_search {
            // Research step - use researcher agent with web_search_tool and crawl_tool
            self.execute_research_step(step, plan_title, completed_steps)
                .await
        } else {
            // Processing step - use coder agent with lua_repl
            self.execute_processing_step(step, plan_title, completed_steps)
                .await
        };

        match result {
            Ok(result) => {
                current_step.status = DeepStepStatus::Completed;
                current_step.result = Some(result);
            }
            Err(e) => {
                log::error!("Step {} failed: {:?}", idx + 1, e);
                current_step.status = DeepStepStatus::Failed;
                current_step.result = Some(format!("Failed: {}", e));
            }
        }

        // Stream updated step
        let step_json = serde_json::to_string(&current_step).unwrap();
        self.stream_step(&step_json).await?;

        Ok(current_step)
    }

    async fn execute_research_step(
        &mut self,
        step: &DeepStep,
        plan_title: &str,
        completed_steps: &[DeepStep],
    ) -> Result<String> {
        // Build completed steps section
        let mut completed_steps_text = String::new();
        for (idx, completed_step) in completed_steps.iter().enumerate() {
            completed_steps_text.push_str(&format!(
                "## Completed Step {}: {}\n<finding>{}</finding>\n\n",
                idx + 1,
                completed_step.description,
                completed_step.result.as_deref().unwrap_or("No result")
            ));
        }

        // Extract title and description from step
        let parts: Vec<&str> = step.description.splitn(2, ": ").collect();
        let (step_title, step_description) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("", step.description.as_str())
        };

        let system_prompt = self
            .prompt
            .render_researcher(
                &self.completion_ctx,
                plan_title,
                &completed_steps_text,
                step_title,
                step_description,
            )
            .context("Failed to render researcher prompt")?;

        // Split the system prompt into two system messages
        // The researcher prompt should have --- separator for two system messages
        let system_messages: Vec<&str> = system_prompt.split("---").collect();
        let first_system_msg = system_messages.get(0).map(|s| s.trim()).unwrap_or(&system_prompt);
        let second_system_msg = if system_messages.len() > 1 {
            system_messages.get(1).map(|s| s.trim()).unwrap_or("")
        } else {
            ""
        };

        // Create tools for researcher
        let web_search_tool = openrouter::Tool {
            name: "web_search_tool".to_string(),
            description: "Use this to perform web search and gather online information.".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The topic to search, expected to be started with What/Where/How..."
                    }
                },
                "required": ["query"]
            }),
        };

        let crawl_tool = openrouter::Tool {
            name: "crawl_tool".to_string(),
            description: "Use this to crawl a url and get a readable content in markdown format.".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The url to crawl."
                    }
                },
                "required": ["url"]
            }),
        };

        let tools = vec![web_search_tool, crawl_tool];

        // Multi-turn conversation with researcher - start with two system messages
        let mut messages = vec![
            openrouter::Message::System(first_system_msg.to_string()),
        ];
        
        if !second_system_msg.is_empty() {
            messages.push(openrouter::Message::System(second_system_msg.to_string()));
        }
        
        messages.push(openrouter::Message::User(step.description.clone()));

        let mut final_response = String::new();
        let max_turns = 5; // Limit conversation turns

        for turn in 0..max_turns {
            let mut res = self
                .ctx
                .openrouter
                .stream(messages.clone(), &self.model, tools.clone())
                .await?;

            let halt = self
                .completion_ctx
                .put_stream((&mut res).map(|resp| resp.map(Into::into)))
                .await?;

            if matches!(halt, StreamEndReason::Halt) {
                log::debug!("The stream was halted");
                break;
            }

            let result = res.get_result();

            self.completion_ctx
                .update_usage(result.usage.cost as f32, result.usage.token as i32);

            // Collect the response
            let response: String = result
                .responses
                .iter()
                .filter_map(|r| match r {
                    openrouter::StreamCompletionResp::ResponseToken(token) => Some(token.as_str()),
                    _ => None,
                })
                .collect();

            // Check if there's a tool call
            if let Some(tool_call) = result.toolcall {
                // Add assistant message with tool call
                messages.push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    arguments: tool_call.args.clone(),
                }));

                // Execute the tool
                let tool_result = self.execute_tool(&tool_call.name, &tool_call.args).await;
                
                let result_text = match tool_result {
                    Ok(result) => result,
                    Err(e) => format!("Error: {}", e),
                };

                // Add tool result message
                messages.push(openrouter::Message::ToolResult(openrouter::MessageToolResult {
                    id: tool_call.id,
                    content: result_text,
                }));

                // Continue conversation
                continue;
            } else {
                // No tool call, this is the final response
                final_response = response;
                break;
            }
        }

        if final_response.is_empty() {
            return Ok(format!(
                "No research results obtained for step: {}",
                step.description
            ));
        }

        Ok(final_response)
    }

    async fn execute_processing_step(
        &mut self,
        step: &DeepStep,
        plan_title: &str,
        completed_steps: &[DeepStep],
    ) -> Result<String> {
        // Build completed steps section
        let mut completed_steps_text = String::new();
        for (idx, completed_step) in completed_steps.iter().enumerate() {
            completed_steps_text.push_str(&format!(
                "## Completed Step {}: {}\n<finding>{}</finding>\n\n",
                idx + 1,
                completed_step.description,
                completed_step.result.as_deref().unwrap_or("No result")
            ));
        }

        // Extract title and description from step
        let parts: Vec<&str> = step.description.splitn(2, ": ").collect();
        let (step_title, step_description) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("", step.description.as_str())
        };

        let system_prompt = self
            .prompt
            .render_coder(
                &self.completion_ctx,
                plan_title,
                &completed_steps_text,
                step_title,
                step_description,
            )
            .context("Failed to render coder prompt")?;

        // Split the system prompt into two system messages if available
        let system_messages: Vec<&str> = system_prompt.split("---").collect();
        let first_system_msg = system_messages.get(0).map(|s| s.trim()).unwrap_or(&system_prompt);
        let second_system_msg = if system_messages.len() > 1 {
            system_messages.get(1).map(|s| s.trim()).unwrap_or("")
        } else {
            ""
        };

        // Create lua_repl tool
        let lua_repl_tool = openrouter::Tool {
            name: "lua_repl".to_string(),
            description: "Use this to execute lua code and do data analysis or calculation. If you want to see the output of a value, you should print it out with `print(...)`. This is visible to the user.".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "The lua code to execute to do further analysis or calculation."
                    }
                },
                "required": ["code"]
            }),
        };

        let tools = vec![lua_repl_tool];

        // Multi-turn conversation with coder - start with system messages
        let mut messages = vec![
            openrouter::Message::System(first_system_msg.to_string()),
        ];
        
        if !second_system_msg.is_empty() {
            messages.push(openrouter::Message::System(second_system_msg.to_string()));
        }
        
        messages.push(openrouter::Message::User(step.description.clone()));

        let mut final_response = String::new();
        let max_turns = 10; // Allow more turns for processing

        for turn in 0..max_turns {
            let mut res = self
                .ctx
                .openrouter
                .stream(messages.clone(), &self.model, tools.clone())
                .await?;

            let halt = self
                .completion_ctx
                .put_stream((&mut res).map(|resp| resp.map(Into::into)))
                .await?;

            if matches!(halt, StreamEndReason::Halt) {
                log::debug!("The stream was halted");
                break;
            }

            let result = res.get_result();

            self.completion_ctx
                .update_usage(result.usage.cost as f32, result.usage.token as i32);

            // Collect the response
            let response: String = result
                .responses
                .iter()
                .filter_map(|r| match r {
                    openrouter::StreamCompletionResp::ResponseToken(token) => Some(token.as_str()),
                    _ => None,
                })
                .collect();

            // Check if there's a tool call
            if let Some(tool_call) = result.toolcall {
                // Add assistant message with tool call
                messages.push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    arguments: tool_call.args.clone(),
                }));

                // Execute the tool
                let tool_result = self.execute_tool(&tool_call.name, &tool_call.args).await;
                
                let result_text = match tool_result {
                    Ok(result) => result,
                    Err(e) => format!("Error: {}", e),
                };

                // Add tool result message
                messages.push(openrouter::Message::ToolResult(openrouter::MessageToolResult {
                    id: tool_call.id,
                    content: result_text,
                }));

                // Continue conversation
                continue;
            } else {
                // No tool call, this is the final response
                final_response = response;
                break;
            }
        }

        if final_response.is_empty() {
            return Ok(format!(
                "No processing results obtained for step: {}",
                step.description
            ));
        }

        Ok(final_response)
    }

    async fn execute_tool(&self, tool_name: &str, args: &str) -> Result<String> {
        match tool_name {
            "web_search_tool" => {
                #[derive(Deserialize)]
                struct WebSearchArgs {
                    query: String,
                }
                let args: WebSearchArgs = serde_json::from_str(args)?;
                let results = self.ctx.web_search_tool.search(&args.query).await?;
                
                let mut output = String::new();
                for (i, result) in results.iter().enumerate().take(10) {
                    output.push_str(&format!(
                        "{}. [{}]({})\n   {}\n\n",
                        i + 1,
                        result.title,
                        result.url,
                        result.description
                    ));
                }
                
                if output.is_empty() {
                    output = "No search results found.".to_string();
                }
                
                Ok(output)
            }
            "crawl_tool" => {
                #[derive(Deserialize)]
                struct CrawlArgs {
                    url: String,
                }
                let args: CrawlArgs = serde_json::from_str(args)?;
                let content = self.ctx.crawl_tool.crawl(&args.url).await?;
                Ok(content)
            }
            "lua_repl" => {
                #[derive(Deserialize)]
                struct LuaArgs {
                    code: String,
                }
                let args: LuaArgs = serde_json::from_str(args)?;
                let result = self.ctx.lua_repl_tool.execute(&args.code).await?;
                Ok(result)
            }
            _ => anyhow::bail!("Unknown tool: {}", tool_name),
        }
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
        &mut self,
        _plan: &DeepPlan,
        step_results: &[DeepStep],
    ) -> Result<DeepReport> {
        let system_prompt = self
            .prompt
            .render_reporter(&self.completion_ctx)
            .context("Failed to render reporter prompt")?;

        // Split the system prompt into two system messages
        // The reporter prompt should have --- separator for two system messages
        let system_messages: Vec<&str> = system_prompt.split("---").collect();
        let first_system_msg = system_messages.get(0).map(|s| s.trim()).unwrap_or(&system_prompt);
        let second_system_msg = if system_messages.len() > 1 {
            system_messages.get(1).map(|s| s.trim()).unwrap_or("")
        } else {
            ""
        };

        // Build user message from step results
        let mut user_context = String::new();
        for (idx, step) in step_results.iter().enumerate() {
            user_context.push_str(&format!("Below are some observations for the research task:\n\n"));
            
            if let Some(result) = &step.result {
                user_context.push_str(&format!("{}\n", result));
            } else {
                user_context.push_str("No results available.\n");
            }
            
            if idx < step_results.len() - 1 {
                user_context.push_str("\n");
            }
        }

        // Create messages with two system prompts
        let mut messages = vec![
            openrouter::Message::System(first_system_msg.to_string()),
        ];
        
        if !second_system_msg.is_empty() {
            messages.push(openrouter::Message::System(second_system_msg.to_string()));
        }
        
        messages.push(openrouter::Message::User(user_context));

        let mut res = self
            .ctx
            .openrouter
            .stream(messages, &self.model, vec![])
            .await?;

        let halt = self
            .completion_ctx
            .put_stream((&mut res).map(|resp| resp.map(Into::into)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            log::debug!("The stream was halted");
        }

        let result = res.get_result();

        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        // Collect the report
        let response: String = result
            .responses
            .iter()
            .filter_map(|r| match r {
                openrouter::StreamCompletionResp::ResponseToken(token) => Some(token.as_str()),
                _ => None,
            })
            .collect();

        Ok(DeepReport { content: response })
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
