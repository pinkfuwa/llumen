use std::sync::Arc;

use anyhow::{Context as _, Result};
use futures_util::future::BoxFuture;
use serde::Deserialize;
use tokio_stream::StreamExt;

use crate::chat::deep_prompt::{CompletedStep, PromptContext};
use crate::chat::process::chat::ChatPipeline;
use crate::chat::processes::deep::helper::{
    get_crawl_tool_def, get_lua_repl_def, get_web_search_tool_def,
};
use crate::chat::{CompletionContext, Context};
use crate::openrouter;

use super::helper::{PlannerResponse, PlannerStep, from_str_error};

/// Deep research agent that orchestrates multiple agents for comprehensive research
pub struct DeepAgent<'a> {
    ctx: Arc<Context>,
    completion_ctx: &'a mut CompletionContext,
    model: openrouter::Model,
    completed_steps: Vec<CompletedStep>,
    plan: PlannerResponse,
    enhanced_prompt: String,
}

impl<'a> DeepAgent<'a> {
    pub fn handoff_tool(
        pipeline: &'a mut ChatPipeline<super::Inner>,
        _toolcall: openrouter::ToolCall,
    ) -> BoxFuture<'a, Result<()>> {
        let model = pipeline.model.clone();
        let ctx = pipeline.ctx.clone();
        let completion_ctx = &mut pipeline.completion_ctx;

        let mut agent = DeepAgent {
            ctx,
            completion_ctx,
            model,
            completed_steps: Vec::new(),
            plan: PlannerResponse::default(),
            enhanced_prompt: String::new(),
        };

        Box::pin(async move {
            macro_rules! handle {
                ($e:ident) => {
                    if let Err(err) = agent.$e().await {
                        agent.completion_ctx.add_error_chunk(err.to_string());
                        return Ok(());
                    }
                };
            }
            handle!(enhance);
            handle!(plan);
            handle!(execute_steps);
            Ok(())
        })
    }

    // Remove the separate run_agent method
    fn get_prompt_context(&self) -> PromptContext {
        use time::UtcDateTime;
        use time::macros::format_description;

        const TIME_FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
            format_description!("[weekday], [hour]:[minute], [day] [month] [year]");

        let time = UtcDateTime::now().format(&TIME_FORMAT).unwrap();
        let locale = self
            .completion_ctx
            .user
            .preference
            .locale
            .clone()
            .unwrap_or_else(|| "en-US".to_string());

        PromptContext {
            time,
            locale,
            max_step_num: 6,
            user_prompt: self
                .completion_ctx
                .latest_user_message()
                .map(|s| s.to_string()),
            plan_title: None,
            completed_steps: None,
            current_step_title: None,
            current_step_description: None,
            enhanced_prompt: None,
        }
    }
    async fn enhance(&mut self) -> Result<()> {
        let original_prompt = self.completion_ctx.latest_user_message().unwrap_or("");

        let prompt_ctx = self.get_prompt_context();
        let system_prompt = self.ctx.deep_prompt.render_prompt_enhancer(&prompt_ctx)?;

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
        let mut prompt_ctx = self.get_prompt_context();
        prompt_ctx.user_prompt = Some(self.enhanced_prompt.clone());

        let system_prompt = self.ctx.deep_prompt.render_planner(&prompt_ctx)?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(self.enhanced_prompt.clone()),
        ];

        let plan_json = {
            let mut stream = self
                .ctx
                .openrouter
                .stream(messages, &self.model, vec![])
                .await?;

            let mut json = String::new();
            while let Some(token) = StreamExt::next(&mut stream).await {
                if let openrouter::StreamCompletionResp::ResponseToken(delta) = token? {
                    json.push_str(&delta);
                    if self
                        .completion_ctx
                        .add_token(crate::chat::Token::ResearchPlan(delta))
                        .is_err()
                    {
                        return Ok(());
                    }
                }
            }
            json
        };

        log::debug!("Plan: {}", &plan_json);
        // Parse the JSON response
        self.plan = from_str_error(&plan_json, "plan")?;

        Ok(())
    }
    async fn execute_steps(&mut self) -> Result<()> {
        // If already has enough context, generate report directly
        if self.plan.has_enough_context {
            self.generate_report().await?;
            return Ok(());
        }

        // Execute each step
        for step in self.plan.steps.clone() {
            self.execute_step(&step).await?;
        }

        // Generate final report
        self.generate_report().await?;

        Ok(())
    }
    async fn execute_step(&mut self, step: &PlannerStep) -> Result<()> {
        let mut prompt_ctx = self.get_prompt_context();
        prompt_ctx.plan_title = Some(self.plan.title.clone());
        prompt_ctx.locale = self.plan.locale.clone();
        prompt_ctx.current_step_title = Some(step.title.clone());
        prompt_ctx.current_step_description = Some(step.description.clone());

        // Build completed steps context using the array
        if !self.completed_steps.is_empty() {
            prompt_ctx.completed_steps = Some(self.completed_steps.clone());
        }

        let (system_prompt, tools) = if step.step_type.starts_with("processing") {
            let tools = vec![get_lua_repl_def()];
            let system_prompt = self.ctx.deep_prompt.render_coder(&prompt_ctx)?;
            (system_prompt, tools)
        } else {
            let mut tools = vec![get_crawl_tool_def()];
            if step.need_search {
                tools.push(get_web_search_tool_def());
            }
            let system_prompt = self.ctx.deep_prompt.render_researcher(&prompt_ctx)?;
            (system_prompt, tools)
        };

        // Render step input using template
        let step_input = self.ctx.deep_prompt.render_step_input(&prompt_ctx)?;
        let step_system_message = self
            .ctx
            .deep_prompt
            .render_step_system_message(&prompt_ctx)?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::System(step_system_message),
            openrouter::Message::User(step_input),
        ];

        // Execute with tool calls
        let mut step_result = String::new();
        let mut current_messages = messages;

        loop {
            let (assistant_text, tool_calls) = {
                let mut stream = self
                    .ctx
                    .openrouter
                    .stream(current_messages.clone(), &self.model, tools.clone())
                    .await?;

                let mut text = String::new();
                let mut calls = Vec::new();

                while let Some(token) = StreamExt::next(&mut stream).await {
                    match token? {
                        openrouter::StreamCompletionResp::ResponseToken(delta) => {
                            text.push_str(&delta);
                            // Stream step output to user
                            if self
                                .completion_ctx
                                .add_token(crate::chat::Token::ResearchStep(delta))
                                .is_err()
                            {
                                return Ok(());
                            }
                        }
                        openrouter::StreamCompletionResp::ToolCall { name, args, id } => {
                            calls.push(openrouter::ToolCall { id, name, args });
                        }
                        openrouter::StreamCompletionResp::Usage { .. } => {
                            // Capture finish reason
                            // The finish reason comes implicitly when the stream ends
                        }
                        _ => {}
                    }
                }

                (text, calls)
            };

            // Check if we have tool calls
            if !tool_calls.is_empty() {
                // Process tool calls
                current_messages.push(openrouter::Message::Assistant(assistant_text.clone()));

                for tool_call in &tool_calls {
                    let result = self.execute_tool(&tool_call.name, &tool_call.args).await?;
                    current_messages.push(openrouter::Message::ToolResult(
                        openrouter::MessageToolResult {
                            id: tool_call.id.clone(),
                            content: result,
                        },
                    ));
                }

                continue;
            } else {
                step_result = assistant_text;
                break;
            }
        }

        self.completed_steps.push(CompletedStep {
            title: step.title.clone(),
            content: step_result,
        });
        Ok(())
    }

    async fn generate_report(&mut self) -> Result<()> {
        let mut prompt_ctx = self.get_prompt_context();
        prompt_ctx.plan_title = Some(self.plan.title.clone());
        prompt_ctx.locale = self.plan.locale.clone();
        prompt_ctx.enhanced_prompt = Some(self.enhanced_prompt.clone());
        prompt_ctx.completed_steps = Some(self.completed_steps.clone());

        let system_prompt = self.ctx.deep_prompt.render_reporter(&prompt_ctx)?;
        let report_input = self.ctx.deep_prompt.render_report_input(&prompt_ctx)?;

        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(report_input),
        ];

        {
            let mut stream = self
                .ctx
                .openrouter
                .stream(messages, &self.model, vec![])
                .await?;

            // Stream the report back to the user
            while let Some(token) = StreamExt::next(&mut stream).await {
                match token? {
                    openrouter::StreamCompletionResp::ResponseToken(delta) => {
                        // Send the delta using ResearchReport token
                        if self
                            .completion_ctx
                            .add_token(crate::chat::Token::ResearchReport(delta))
                            .is_err()
                        {
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }

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
}
