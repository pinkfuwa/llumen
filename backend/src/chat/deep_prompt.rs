use anyhow::Result;
use minijinja::Environment;
use serde::Serialize;
use time::UtcDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::chat::CompletionContext;

pub struct PromptContext {}

pub struct DeepPrompt {
    env: Environment<'static>,
}

#[derive(Serialize)]
struct DeepRenderingContext<'a> {
    time: String,
    locale: &'a str,
    max_step_num: usize,
    user_prompt: Option<&'a str>,
    // For researcher: completed steps and current step
    plan_title: Option<&'a str>,
    completed_steps: Option<&'a str>,
    current_step_title: Option<&'a str>,
    current_step_description: Option<&'a str>,
}

const TIME_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[weekday], [hour]:[minute], [day] [month] [year]");

impl DeepPrompt {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.add_template(
            "deep_coordinator",
            include_str!("../../../prompts/deepresearch/coordinator.md"),
        )
        .unwrap();
        env.add_template(
            "deep_prompt_enhancer",
            include_str!("../../../prompts/deepresearch/prompt_enhancer.md"),
        )
        .unwrap();
        env.add_template(
            "deep_planner",
            include_str!("../../../prompts/deepresearch/planner.md"),
        )
        .unwrap();
        env.add_template(
            "deep_researcher",
            include_str!("../../../prompts/deepresearch/researcher.md"),
        )
        .unwrap();
        env.add_template(
            "deep_coder",
            include_str!("../../../prompts/deepresearch/coder.md"),
        )
        .unwrap();
        env.add_template(
            "deep_reporter",
            include_str!("../../../prompts/deepresearch/reporter.md"),
        )
        .unwrap();
        Self { env }
    }

    pub fn render_coordinator(&self, ctx: &PromptContext) -> Result<String> {
        todo!()
    }

    pub fn render_prompt_enhancer(&self, ctx: &PromptContext) -> Result<String> {
        todo!()
    }

    pub fn render_planner(&self, ctx: &PromptContext) -> Result<String> {
        todo!()
    }

    pub fn render_researcher(&self, ctx: &PromptContext) -> Result<String> {
        todo!()
    }

    pub fn render_coder(&self, ctx: &PromptContext) -> Result<String> {
        todo!()
    }

    pub fn render_reporter(&self, ctx: &PromptContext) -> Result<String> {
        todo!()
    }
}
