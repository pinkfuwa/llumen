use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use protocol::*;
use serde::Deserialize;
use tokio_stream::StreamExt;

use super::helper::*;
use crate::chat::context::StreamEndReason;
use crate::chat::converter::*;
use crate::chat::deep_prompt::{CompletedStep, ReportInputContext, StepInputContext};
use crate::chat::process::chat::ChatPipeline;
use crate::chat::{CompletionContext, Context, Token};
use crate::openrouter;
use crate::utils::model::{ModelCapability, ModelChecker};

/// Deep research agent that orchestrates multiple agents for comprehensive research
pub struct DeepAgent<'a> {
    ctx: Arc<Context>,
    completion_ctx: &'a mut CompletionContext,
    model: openrouter::Model,
    state: Option<Deep>,
    enhanced_prompt: String,
}

impl<'a> DeepAgent<'a> {
    pub async fn handoff_tool(
        pipeline: &'a mut ChatPipeline<super::Inner>,
        _toolcall: Vec<openrouter::ToolCall>,
    ) -> Result<()> {
        let model = pipeline.model.clone();
        let ctx = pipeline.ctx.clone();
        let completion_ctx = &mut pipeline.completion_ctx;

        let mut agent = DeepAgent {
            ctx,
            completion_ctx,
            model,
            state: None,
            enhanced_prompt: String::new(),
        };

        macro_rules! handle {
            ($e:ident) => {
                if let Err(err) = agent.$e().await {
                    agent.completion_ctx.add_error(err.to_string());
                    return Ok(());
                }
            };
        }
        handle!(enhance);
        handle!(plan);
        handle!(execute_steps);
        Ok(())
    }

    // Remove the separate run_agent method
    fn get_locale<'b>(&'b self) -> &'b str {
        self.completion_ctx
            .user
            .preference
            .locale
            .as_ref()
            .map(|x| x.as_str())
            .unwrap_or_else(|| "en-US")
    }

    fn get_model_with_schema(
        &self,
        schema_name: &str,
        schema: serde_json::Value,
    ) -> openrouter::Model {
        // Check if model supports JSON structured output
        let supports_json = ModelConfig::from_toml(&self.completion_ctx.model.config)
            .ok()
            .map(|config| config.is_json_capable())
            .unwrap_or(false);

        if supports_json {
            log::debug!(
                "Model supports JSON, using structured output for {}",
                schema_name
            );
            openrouter::Model::builder(self.model.id.clone())
                .temperature(self.model.temperature.unwrap_or(0.7))
                .json_schema(schema_name, schema)
                .build()
        } else {
            log::debug!("Model does not support JSON, using regular output");
            self.model.clone()
        }
    }
    async fn enhance(&mut self) -> Result<()> {
        let original_prompt = self.completion_ctx.latest_user_message().unwrap_or("");

        let system_prompt = self
            .ctx
            .deep_prompt
            .render_prompt_enhancer(self.get_locale())?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(original_prompt.to_string()),
        ];

        let enhanced_text = {
            let mut stream = self
                .ctx
                .openrouter
                .stream(messages, &self.model, vec![])
                .await?;

            let mut text = String::new();
            while let Some(token) = StreamExt::next(&mut stream).await {
                match token? {
                    openrouter::StreamCompletionResp::ResponseToken(delta) => {
                        text.push_str(&delta);
                    }
                    _ => {}
                }
            }
            text
        };

        if let Some(start) = enhanced_text.find("<enhanced_prompt>") {
            if let Some(end) = enhanced_text.find("</enhanced_prompt>") {
                let start_pos = start + "<enhanced_prompt>".len();
                self.enhanced_prompt = enhanced_text[start_pos..end].trim().to_string();
            } else {
                self.enhanced_prompt = enhanced_text;
            }
        } else {
            self.enhanced_prompt = original_prompt.to_string();
        }

        log::debug!("Enhanced Prompt: {}", self.enhanced_prompt);

        Ok(())
    }
    async fn plan(&mut self) -> Result<()> {
        let system_prompt = self.ctx.deep_prompt.render_planner(self.get_locale())?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(self.enhanced_prompt.clone()),
        ];

        // Define the JSON schema for the planner response
        let plan_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "locale": {
                    "type": "string",
                    "description": "The detected language locale"
                },
                "has_enough_context": {
                    "type": "boolean",
                    "description": "Whether the current context is sufficient to answer the query"
                },
                "thought": {
                    "type": "string",
                    "description": "Internal reasoning about the plan"
                },
                "title": {
                    "type": "string",
                    "description": "Title of the research plan"
                },
                "steps": {
                    "type": "array",
                    "description": "List of research steps",
                    "items": {
                        "type": "object",
                        "properties": {
                            "need_search": {
                                "type": "boolean",
                                "description": "Whether this step requires web search"
                            },
                            "title": {
                                "type": "string",
                                "description": "Title of the step"
                            },
                            "description": {
                                "type": "string",
                                "description": "Detailed description of what to do in this step"
                            },
                            "step_type": {
                                "type": "string",
                                "enum": ["code", "research"],
                                "description": "Type of step: code or research"
                            }
                        },
                        "required": ["need_search", "title", "description", "step_type"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["locale", "has_enough_context", "thought", "title", "steps"],
            "additionalProperties": false
        });

        let model_with_schema = self.get_model_with_schema("planner_response", plan_schema);

        let mut stream = self
            .ctx
            .openrouter
            .stream(messages, &model_with_schema, vec![])
            .await?;

        let halt = self
            .completion_ctx
            .put_stream((&mut stream).map(|resp| resp.map(openrouter_to_buffer_token_deep_plan)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            bail!("Plan generation interrupted");
        }

        let result = stream.get_result();
        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        let plan_json = result.get_text();

        log::debug!("Plan: {}", &plan_json);
        // Parse the JSON response
        self.state = Some(from_str_error::<PlannerResponse>(&plan_json, "plan")?.into());

        Ok(())
    }
    async fn execute_steps(&mut self) -> Result<()> {
        let plan = self.state.as_mut().unwrap();
        // If already has enough context, generate report directly
        if plan.has_enough_context {
            self.generate_report().await?;
            return Ok(());
        }

        // Execute each step
        for i in 0..plan.steps.len() {
            self.execute_step(i).await?;
        }

        // Generate final report
        self.generate_report().await?;

        Ok(())
    }
    async fn execute_step(&mut self, step_idx: usize) -> Result<()> {
        let locale = self.get_locale();
        let plan = self.state.as_ref().unwrap();
        let step = plan.steps.get(step_idx).unwrap();

        let (system_prompt, tools) = if step.kind == StepKind::Code {
            let tools = vec![get_lua_repl_def()];
            let system_prompt = self.ctx.deep_prompt.render_coder(locale)?;
            (system_prompt, tools)
        } else {
            let mut tools = vec![get_crawl_tool_def()];
            if step.need_search {
                tools.push(get_web_search_tool_def());
            }
            let system_prompt = self.ctx.deep_prompt.render_researcher(locale)?;
            (system_prompt, tools)
        };

        let completed_steps = plan
            .steps
            .iter()
            .filter_map(|s| {
                if !s.progress.is_empty() {
                    return None;
                }
                Some(CompletedStep {
                    title: &s.title,
                    content: s.progress.last().and_then(AssistantChunk::as_text)?,
                })
            })
            .collect::<Vec<_>>();

        let step_input_ctx = StepInputContext {
            locale: locale,
            plan_title: plan.title.as_str(),
            completed_steps,
            current_step_title: step.title.as_str(),
            current_step_description: step.description.as_str(),
        };
        // Render step input using template
        let step_input = self.ctx.deep_prompt.render_step_input(&step_input_ctx)?;
        let step_system_message = self
            .ctx
            .deep_prompt
            .render_step_system_message(self.get_locale())?;

        let mut progress = Vec::new();

        let mut messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::System(step_system_message),
            openrouter::Message::User(step_input),
        ];

        self.completion_ctx
            .add_token_force(Token::DeepStepStart(step_idx as i32));

        loop {
            let mut stream = self
                .ctx
                .openrouter
                .stream(messages.clone(), &self.model, tools.clone())
                .await?;

            let halt = self
                .completion_ctx
                .put_stream(
                    (&mut stream).map(|resp| resp.map(openrouter_to_buffer_token_deep_step)),
                )
                .await?;

            if matches!(halt, StreamEndReason::Halt) {
                bail!("step interrupted");
            }

            let mut result = stream.get_result();
            self.completion_ctx
                .update_usage(result.usage.cost as f32, result.usage.token as i32);

            let tool_calls = std::mem::take(&mut result.toolcalls);

            let assistant_text = result.get_text();

            progress.extend(openrouter_stream_to_assitant_chunk(&result.responses));
            if tool_calls.is_empty() {
                break;
            }

            messages.push(openrouter::Message::Assistant(assistant_text.clone()));

            for tool_call in tool_calls {
                messages.push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    arguments: tool_call.args.clone(),
                }));

                self.completion_ctx
                    .add_token_force(Token::DeepStepToolCall {
                        name: tool_call.name.clone(),
                        arg: tool_call.args.clone(),
                    });

                let result = self.execute_tool(&tool_call.name, &tool_call.args).await?;

                messages.push(openrouter::Message::ToolResult(
                    openrouter::MessageToolResult {
                        id: tool_call.id,
                        content: result.clone(),
                    },
                ));

                self.completion_ctx
                    .add_token_force(Token::DeepStepToolResult(result))
            }
        }

        self.state.as_mut().unwrap().steps[step_idx].progress = progress;

        Ok(())
    }

    async fn generate_report(&mut self) -> Result<()> {
        let plan = self.state.as_ref().unwrap();

        let completed_steps = plan
            .steps
            .iter()
            .filter_map(|s| {
                if !s.progress.is_empty() {
                    return None;
                }
                Some(CompletedStep {
                    title: &s.title,
                    content: s.progress.last().and_then(AssistantChunk::as_text)?,
                })
            })
            .collect::<Vec<_>>();

        let report_input_ctx = ReportInputContext {
            locale: self.get_locale(),
            plan_title: plan.title.as_str(),
            completed_steps,
            enhanced_prompt: self.enhanced_prompt.as_str(),
        };

        let system_prompt = self.ctx.deep_prompt.render_reporter(&self.get_locale())?;
        let report_input = self
            .ctx
            .deep_prompt
            .render_report_input(&report_input_ctx)?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(report_input),
        ];

        let mut stream = self
            .ctx
            .openrouter
            .stream(messages, &self.model, vec![])
            .await?;

        let halt = self
            .completion_ctx
            .put_stream((&mut stream).map(|resp| resp.map(openrouter_to_buffer_token_deep_report)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            bail!("reporter interrupted");
        }

        let result = stream.get_result();
        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);
        let text = result.get_text();

        let chunks = self.completion_ctx.message.inner.as_assistant().unwrap();
        chunks.push(AssistantChunk::DeepAgent(self.state.take().unwrap()));
        chunks.push(AssistantChunk::Text(text));

        Ok(())
    }
    async fn execute_tool(&self, tool_name: &str, args: &str) -> Result<String> {
        log::debug!("Running tool({}), arg: {}", tool_name, args);
        match tool_name {
            "web_search_tool" => {
                #[derive(Deserialize)]
                struct WebSearchArgs {
                    query: String,
                }
                let args: Option<WebSearchArgs> = serde_json::from_str(args).ok();
                if args.is_none() {
                    return Ok("Invalid arguments for web_search_tool".to_string());
                }
                let args = args.unwrap();
                match self.ctx.web_search_tool.search(&args.query).await {
                    Ok(results) => {
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
                    Err(e) => {
                        // Return error as string so agent can see it and potentially recover
                        log::warn!("Web search error: {}", e);
                        Ok(format!("Error: {}", e))
                    }
                }
            }
            "crawl_tool" => {
                #[derive(Deserialize)]
                struct CrawlArgs {
                    url: String,
                }
                let args: Option<CrawlArgs> = serde_json::from_str(args).ok();
                if args.is_none() {
                    return Ok("Invaild arguments".to_string());
                }
                let args = args.unwrap();
                match self.ctx.crawl_tool.crawl(&args.url).await {
                    Ok(content) => Ok(content),
                    Err(e) => {
                        // Return error as string so agent can see it and potentially recover
                        log::warn!("Crawl error for URL '{}': {}", args.url, e);
                        Ok(format!("Error: {}", e))
                    }
                }
            }
            "lua_repl" => {
                #[derive(Deserialize)]
                struct LuaArgs {
                    code: String,
                }
                let args: Option<LuaArgs> = serde_json::from_str(args).ok();
                if args.is_none() {
                    return Ok("Invaild arguments".to_string());
                }
                let args = args.unwrap();
                match self.ctx.lua_repl_tool.execute(&args.code).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        // Return error as string so agent can see it and potentially recover
                        log::warn!("Lua execution error: {}", e);
                        Ok(format!("Error: {}", e))
                    }
                }
            }
            _ => anyhow::bail!("Unknown tool: {}", tool_name),
        }
    }
}
