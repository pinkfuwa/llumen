use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use protocol::*;
use serde::Deserialize;
use tokio_stream::StreamExt;

use super::helper::*;
use crate::chat::context::StreamEndReason;
use crate::chat::converter::*;
use crate::chat::prompt::{CompletedStep, ReportInputContext, StepInputContext};
use crate::chat::tools::get_lua_repl_def;
use crate::chat::{CompletionSession, Context, Token, TokenSink};
use crate::openrouter::{self, ReasoningEffort};

/// Input context for DeepAgent, extracted from CompletionSession.
pub struct DeepAgentInput {
    pub user_message: String,
    pub locale: String,
    pub model: openrouter::Model,
}

/// Deep research agent that orchestrates multiple agents for comprehensive
/// research
pub struct DeepAgent {
    ctx: Arc<Context>,
    input: DeepAgentInput,
    state: Option<Deep>,
    enhanced_prompt: String,
}

impl DeepAgent {
    pub async fn handoff_tool_static(
        ctx: &Arc<Context>,
        completion_ctx: &mut CompletionSession,
        _toolcall: Vec<openrouter::ToolCall>,
    ) -> Result<()> {
        use crate::utils::model::ModelChecker;
        use protocol::ModelConfig;

        let model = <ModelConfig as ModelChecker>::from_toml(&completion_ctx.model.config)
            .context("Failed to get model config")?;
        let model: openrouter::Model = model.into();

        let input = DeepAgentInput {
            user_message: completion_ctx
                .latest_user_message()
                .unwrap_or("")
                .to_string(),
            locale: completion_ctx
                .user
                .preference
                .locale
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("en-US")
                .to_string(),
            model,
        };

        let mut agent = DeepAgent {
            ctx: ctx.clone(),
            input,
            state: None,
            enhanced_prompt: String::new(),
        };

        if let Err(err) = agent.run(completion_ctx).await {
            completion_ctx.add_error(err.to_string());
        }

        Ok(())
    }

    /// Run the full deep research pipeline: enhance → plan → execute steps →
    /// report.
    async fn run(&mut self, session: &mut CompletionSession) -> Result<()> {
        self.enhance(session).await?;
        self.plan(session).await?;

        let (deep_state, final_text) = self.execute_steps_and_report(session).await?;

        // Store final chunks
        let chunks = session.message.inner.as_assistant().unwrap();
        chunks.push(AssistantChunk::DeepAgent(deep_state));
        chunks.push(AssistantChunk::Text(final_text));

        Ok(())
    }

    fn get_locale(&self) -> &str {
        &self.input.locale
    }

    async fn enhance(&mut self, sink: &mut impl TokenSink) -> Result<()> {
        let original_prompt = &self.input.user_message;

        let system_prompt = self.ctx.prompt.render_prompt_enhancer(self.get_locale())?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(original_prompt.to_string()),
        ];

        let enhanced_text = {
            let model = openrouter::ModelBuilder::from_model(&self.input.model).build();

            let mut stream: openrouter::StreamCompletion = self
                .ctx
                .openrouter
                .stream(model, messages, openrouter::CompletionOption::default())
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
    async fn plan(&mut self, sink: &mut impl TokenSink) -> Result<()> {
        let system_prompt = self.ctx.prompt.render_planner(self.get_locale())?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(self.enhanced_prompt.clone()),
        ];

        let model = openrouter::ModelBuilder::from_model(&self.input.model).build();

        let result = self
            .ctx
            .openrouter
            .structured::<PlannerResponse>(messages, model, openrouter::CompletionOption::default())
            .await?;

        // TODO: since we decide to remove streaming plan, we should also remove support
        // in frontend
        sink.add_token(Token::DeepPlan(
            serde_json::to_string(&result.response).unwrap(),
        ));

        sink.update_usage(result.price as f32, result.token as i32);

        self.state = Some(result.response.into());

        Ok(())
    }
    async fn execute_steps_and_report(
        &mut self,
        sink: &mut impl TokenSink,
    ) -> Result<(Deep, String)> {
        let plan = self.state.as_mut().unwrap();
        // If already has enough context, generate report directly
        if plan.has_enough_context {
            return self.generate_report(sink).await;
        }

        // Execute each step
        for i in 0..plan.steps.len() {
            self.execute_step(i, sink).await?;
        }

        // Generate final report
        self.generate_report(sink).await
    }
    async fn execute_step(&mut self, step_idx: usize, sink: &mut impl TokenSink) -> Result<()> {
        let locale = self.get_locale();
        let plan = self.state.as_ref().unwrap();
        let step = plan.steps.get(step_idx).unwrap();

        let (system_prompt, tools) = if step.kind == StepKind::Code {
            let tools = vec![get_lua_repl_def()];
            let system_prompt = self.ctx.prompt.render_coder(locale)?;
            (system_prompt, tools)
        } else {
            let tools = self.ctx.tools.for_deep_mode(step.need_search);
            let system_prompt = self.ctx.prompt.render_researcher(locale)?;
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
        let step_input = self.ctx.prompt.render_step_input(&step_input_ctx)?;
        let step_system_message = self
            .ctx
            .prompt
            .render_step_system_message(self.get_locale())?;

        let mut progress = Vec::new();

        let mut messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::System(step_system_message),
            openrouter::Message::User(step_input),
        ];

        sink.add_token(Token::DeepStepStart(step_idx as i32));

        loop {
            let model = openrouter::ModelBuilder::from_model(&self.input.model).build();
            let option = openrouter::CompletionOption::tools(&tools);
            let mut stream: openrouter::StreamCompletion = self
                .ctx
                .openrouter
                .stream(model, messages.clone(), option)
                .await?;

            let halt = sink
                .put_stream(
                    (&mut stream).map(|resp| resp.map(openrouter_to_buffer_token_deep_step)),
                )
                .await?;

            if matches!(halt, StreamEndReason::Halt) {
                bail!("step interrupted");
            }

            let mut result = stream.get_result();
            sink.update_usage(result.usage.cost as f32, result.usage.token as i32);

            let tool_calls = std::mem::take(&mut result.toolcalls);

            let assistant_text = result.get_text();

            progress.extend(openrouter_stream_to_assitant_chunk(&result.responses));
            if tool_calls.is_empty() {
                break;
            }

            messages.push(openrouter::Message::Assistant {
                content: assistant_text.clone(),
                annotations: None,
                reasoning_details: None,
                images: Vec::new(),
            });

            for tool_call in tool_calls {
                messages.push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    arguments: tool_call.args.clone(),
                }));

                sink.add_token(Token::DeepStepToolCall {
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

                sink.add_token(Token::DeepStepToolResult(result))
            }
        }

        self.state.as_mut().unwrap().steps[step_idx].progress = progress;

        Ok(())
    }

    async fn generate_report(&mut self, sink: &mut impl TokenSink) -> Result<(Deep, String)> {
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

        let system_prompt = self.ctx.prompt.render_reporter(&self.get_locale())?;
        let report_input = self.ctx.prompt.render_report_input(&report_input_ctx)?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(report_input),
        ];

        let model = openrouter::ModelBuilder::from_model(&self.input.model).build();
        let option = openrouter::CompletionOption::builder()
            .reasoning_effort(ReasoningEffort::Auto)
            .build();
        let mut stream: openrouter::StreamCompletion =
            self.ctx.openrouter.stream(model, messages, option).await?;

        let halt = sink
            .put_stream((&mut stream).map(|resp| resp.map(openrouter_to_buffer_token_deep_report)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            bail!("reporter interrupted");
        }

        let result = stream.get_result();
        sink.update_usage(result.usage.cost as f32, result.usage.token as i32);
        let text = result.get_text();

        Ok((self.state.take().unwrap(), text))
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
                match self.ctx.tools.web_search.search(&args.query).await {
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
                match self.ctx.tools.crawl.crawl(&args.url).await {
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
                match self.ctx.tools.lua_repl.execute(&args.code).await {
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
