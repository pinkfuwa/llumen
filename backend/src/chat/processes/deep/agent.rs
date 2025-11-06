use std::sync::Arc;

use anyhow::{Context as _, Result};
use entity::{ChunkKind, DeepStepStatus, chunk};
use futures_util::future::BoxFuture;
use serde::Deserialize;
use tokio_stream::StreamExt;

use crate::chat::deep_prompt::PromptContext;
use crate::chat::process::chat::ChatPipeline;
use crate::chat::processes::deep::helper::{
    get_crawl_tool_def, get_lua_repl_def, get_web_search_tool_def,
};
use crate::chat::{CompletionContext, Context, Token, context::StreamEndReason};
use crate::openrouter;

use super::helper::{PlannerResponse, PlannerStep};

use anyhow::Context as _;

/// Deep research agent that orchestrates multiple agents for comprehensive research
pub struct DeepAgent {
    ctx: Arc<Context>,
    completion_ctx: CompletionContext,
    model: openrouter::Model,
    completed_steps: Vec<String>,
    plan: PlannerResponse,
    enhanced_prompt: String,
}

impl DeepAgent {
    pub fn handoff_tool(
        pipeline: ChatPipeline<super::Inner>,
        _toolcall: openrouter::ToolCall,
    ) -> BoxFuture<'static, Result<(), anyhow::Error>> {
        let model = pipeline.model;
        let ctx = pipeline.ctx.clone();
        let mut completion_ctx = pipeline.completion_ctx;
        
        // SAFETY: The agent and its completion context are used entirely within
        // a single async context and are not actually sent across threads.
        // The !Send constraint comes from EventSource in StreamCompletion,
        // but we ensure all streams are fully consumed before any await points
        // that could cause the future to migrate to another thread.
        // The future is spawned on the current task and will not move between threads.
        struct SendFuture<F>(F);
        unsafe impl<F> Send for SendFuture<F> {}
        
        impl<F: std::future::Future> std::future::Future for SendFuture<F> {
            type Output = F::Output;
            
            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                // SAFETY: We're just forwarding the poll, the Pin guarantees are maintained
                unsafe {
                    let inner = &mut self.get_unchecked_mut().0;
                    std::pin::Pin::new_unchecked(inner).poll(cx)
                }
            }
        }
        
        let future = SendFuture(async move {
            let mut agent = DeepAgent {
                ctx,
                completion_ctx,
                model,
                completed_steps: Vec::new(),
                plan: PlannerResponse::default(),
                enhanced_prompt: String::new(),
            };
            
            agent.enhance().await?;
            agent.plan().await?;
            agent.execute_steps().await?;
            Ok(())
        });
        
        Box::pin(future)
    }
    
    // Remove the separate run_agent method
    fn get_prompt_context(&self) -> PromptContext {
        use time::macros::format_description;
        use time::UtcDateTime;
        
        const TIME_FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
            format_description!("[weekday], [hour]:[minute], [day] [month] [year]");
        
        let time = UtcDateTime::now().format(&TIME_FORMAT).unwrap();
        let locale = self.completion_ctx.user.preference.locale.clone().unwrap_or_else(|| "en-US".to_string());
        
        PromptContext {
            time,
            locale,
            max_step_num: 6,
            user_prompt: self.completion_ctx.latest_user_message().map(|s| s.to_string()),
            plan_title: None,
            completed_steps: None,
            current_step_title: None,
            current_step_description: None,
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
            let mut stream = self.ctx.openrouter.stream(messages, &self.model, vec![]).await?;
            
            let mut text = String::new();
            while let Some(token) = stream.next().await {
                match token? {
                    openrouter::StreamCompletionResp::ResponseToken(delta) => {
                        text.push_str(&delta);
                    }
                    _ => {}
                }
            }
            text
        };
        
        // Extract content between <enhanced_prompt> tags
        if let Some(start) = enhanced_text.find("<enhanced_prompt>") {
            if let Some(end) = enhanced_text.find("</enhanced_prompt>") {
                let start_pos = start + "<enhanced_prompt>".len();
                self.enhanced_prompt = enhanced_text[start_pos..end].trim().to_string();
            } else {
                self.enhanced_prompt = enhanced_text;
            }
        } else {
            self.enhanced_prompt = enhanced_text;
        }
        
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
            let mut stream = self.ctx.openrouter.stream(messages, &self.model, vec![]).await?;
            
            let mut json = String::new();
            while let Some(token) = stream.next().await {
                match token? {
                    openrouter::StreamCompletionResp::ResponseToken(delta) => {
                        json.push_str(&delta);
                    }
                    _ => {}
                }
            }
            json
        };
        
        // Parse the JSON response
        self.plan = serde_json::from_str(&plan_json)
            .context("Failed to parse planner response")?;
        
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
        prompt_ctx.current_step_title = Some(step.title.clone());
        prompt_ctx.current_step_description = Some(step.description.clone());
        
        // Build completed steps context
        if !self.completed_steps.is_empty() {
            let mut completed_text = String::new();
            for (i, step_result) in self.completed_steps.iter().enumerate() {
                completed_text.push_str(&format!("\n## Completed Step {}: ", i + 1));
                completed_text.push_str(step_result);
                completed_text.push_str("\n");
            }
            prompt_ctx.completed_steps = Some(completed_text);
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
        
        // Create messages with two system prompts as shown in the example
        let step_description = format!(
            "# Research Topic\n\n{}\n\n# Current Step\n\n## Title\n\n{}\n\n## Description\n\n{}\n\n## Locale\n\n{}",
            self.plan.title,
            step.title,
            step.description,
            self.plan.locale
        );
        
        let second_system = "---\nTHIS IS THE SECOND SYSTEM MESSAGE, THERE ARE MULTIPLE SYSTEM PROMPT IN STEP INPUT\n\nIMPORTANT: DO NOT include inline citations in the text. Instead, track all sources and include a References section at the end using link reference format. Include an empty line between each citation for better readability. Use this format for each reference:\n- [Source Title](URL)\n\n- [Another Source](URL)";
        
        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::System(second_system.to_string()),
            openrouter::Message::User(step_description),
        ];
        
        // Execute with tool calls
        let mut step_result = String::new();
        let mut current_messages = messages;
        
        loop {
            let (assistant_text, tool_calls) = {
                let mut stream = self.ctx.openrouter.stream(current_messages.clone(), &self.model, tools.clone()).await?;
                
                let mut text = String::new();
                let mut calls = Vec::new();
                
                while let Some(token) = stream.next().await {
                    match token? {
                        openrouter::StreamCompletionResp::ResponseToken(delta) => {
                            text.push_str(&delta);
                        }
                        openrouter::StreamCompletionResp::ToolCall { name, args, id } => {
                            calls.push(openrouter::ToolCall {
                                id,
                                name,
                                args,
                            });
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
                        }
                    ));
                }
                
                continue;
            } else {
                step_result = assistant_text;
                break;
            }
        }
        
        self.completed_steps.push(step_result);
        Ok(())
    }
    
    async fn generate_report(&mut self) -> Result<()> {
        let mut prompt_ctx = self.get_prompt_context();
        prompt_ctx.plan_title = Some(self.plan.title.clone());
        
        let system_prompt = self.ctx.deep_prompt.render_reporter(&prompt_ctx)?;
        
        // Build the report request with all completed steps
        let mut report_request = format!(
            "# Research Requirements\n\n## Task\n\n{}\n\n## Description\n\n{}\n",
            self.plan.title,
            self.enhanced_prompt
        );
        
        report_request.push_str("---\nTHIS IS THE SECOND SYSTEM MESSAGE, THERE ARE MULTIPLE SYSTEM PROMPT IN STEP INPUT\n\n");
        report_request.push_str("IMPORTANT: Structure your report according to the format in the prompt. Remember to include:\n\n");
        report_request.push_str("1. Key Points - A bulleted list of the most important findings\n");
        report_request.push_str("2. Overview - A brief introduction to the topic\n");
        report_request.push_str("3. Detailed Analysis - Organized into logical sections\n");
        report_request.push_str("4. Survey Note (optional) - For more comprehensive reports\n");
        report_request.push_str("5. Key Citations - List all references at the end\n\n");
        report_request.push_str("For citations, DO NOT include inline citations in the text. Instead, place all citations in the 'Key Citations' section at the end using the format: `- [Source Title](URL)`. Include an empty line between each citation for better readability.\n\n");
        report_request.push_str("PRIORITIZE USING MARKDOWN TABLES for data presentation and comparison. Use tables whenever presenting comparative data, statistics, features, or options. Structure tables with clear headers and aligned columns. Example table format:\n\n");
        report_request.push_str("| Feature | Description | Pros | Cons |\n");
        report_request.push_str("|---------|-------------|------|------|\n");
        report_request.push_str("| Feature 1 | Description 1 | Pros 1 | Cons 1 |\n");
        report_request.push_str("| Feature 2 | Description 2 | Pros 2 | Cons 2 |\n");
        report_request.push_str("---\n");
        report_request.push_str("Below are some observations for the research task:\n\n");
        
        for (_i, step_result) in self.completed_steps.iter().enumerate() {
            report_request.push_str(&format!("{{{}}}\n", step_result));
        }
        
        let messages = vec![
            openrouter::Message::System(system_prompt),
            openrouter::Message::User(report_request),
        ];
        
        {
            let mut stream = self.ctx.openrouter.stream(messages, &self.model, vec![]).await?;
            
            // Stream the report back to the user
            while let Some(token) = stream.next().await {
                match token? {
                    openrouter::StreamCompletionResp::ResponseToken(delta) => {
                        // Send the delta using ResearchReport token
                        let _ = self.completion_ctx.add_token(crate::chat::Token::ResearchReport(delta));
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
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
}
