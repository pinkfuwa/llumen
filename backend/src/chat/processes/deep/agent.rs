use std::sync::Arc;

use anyhow::{Context as _, Result};
use entity::{ChunkKind, DeepStepStatus, chunk};
use futures_util::future::BoxFuture;
use serde::Deserialize;

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
        toolcall: openrouter::ToolCall,
    ) -> BoxFuture<'static, Result<(), anyhow::Error>> {
        let model = pipeline.model;
        let mut agent = Self {
            ctx: pipeline.ctx.clone(),
            completion_ctx: pipeline.completion_ctx,
            model,
            completed_steps: Vec::new(),
            plan: PlannerResponse::default(),
            enhanced_prompt: String::new(),
        };
        Box::pin(async move {
            agent.enhance().await?;
            agent.plan().await?;
            agent.execute_steps().await?;
            Ok(())
        })
    }
    fn get_prompt_context(&self) -> PromptContext {
        todo!()
    }
    async fn enhance(&mut self) -> Result<()> {
        let original_prompt = self.completion_ctx.latest_user_message();
        todo!("run with openrouter");
        self.enhanced_prompt = todo!("remove xml tag");
        Ok(())
    }
    async fn plan(&mut self) -> Result<()> {
        self.plan = todo!();
        Ok(())
    }
    async fn execute_steps(&mut self) -> Result<()> {
        todo!()
    }
    async fn execute_step(&mut self, step: &PlannerStep) -> Result<()> {
        if step.step_type.starts_with("processing") {
            let tool = vec![get_lua_repl_def()];
            // run processor(coder) with openrouter
        } else {
            let mut tool = vec![get_crawl_tool_def()];
            if step.need_search {
                tool.push(get_web_search_tool_def());
            }
            // run researcher with openrouter
        }
        todo!()
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
