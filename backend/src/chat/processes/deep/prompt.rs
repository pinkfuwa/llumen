use anyhow::Result;
use minijinja::Environment;
use serde::Serialize;
use time::UtcDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::chat::CompletionContext;

#[derive(Debug, Clone, Copy)]
pub enum DeepPromptKind {
    Coordinator,
    PromptEnhancer,
    Planner,
    Researcher,
    Reporter,
}

impl DeepPromptKind {
    fn as_str(&self) -> &'static str {
        match self {
            DeepPromptKind::Coordinator => "deep_coordinator",
            DeepPromptKind::PromptEnhancer => "deep_prompt_enhancer",
            DeepPromptKind::Planner => "deep_planner",
            DeepPromptKind::Researcher => "deep_researcher",
            DeepPromptKind::Reporter => "deep_reporter",
        }
    }
}

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
            include_str!("../../../../../prompts/deepresearch/coordinator.md"),
        )
        .unwrap();
        env.add_template(
            "deep_prompt_enhancer",
            include_str!("../../../../../prompts/deepresearch/prompt_enhancer.md"),
        )
        .unwrap();
        env.add_template(
            "deep_planner",
            include_str!("../../../../../prompts/deepresearch/planner.md"),
        )
        .unwrap();
        env.add_template(
            "deep_researcher",
            include_str!("../../../../../prompts/deepresearch/researcher.md"),
        )
        .unwrap();
        env.add_template(
            "deep_reporter",
            include_str!("../../../../../prompts/deepresearch/reporter.md"),
        )
        .unwrap();
        Self { env }
    }

    pub fn render_coordinator(&self, ctx: &CompletionContext) -> Result<String> {
        self.render_basic(DeepPromptKind::Coordinator, ctx)
    }

    pub fn render_prompt_enhancer(&self, ctx: &CompletionContext) -> Result<String> {
        self.render_basic(DeepPromptKind::PromptEnhancer, ctx)
    }

    pub fn render_planner(&self, ctx: &CompletionContext) -> Result<String> {
        self.render_basic(DeepPromptKind::Planner, ctx)
    }

    pub fn render_researcher(
        &self,
        ctx: &CompletionContext,
        plan_title: &str,
        completed_steps: &str,
        current_step_title: &str,
        current_step_description: &str,
    ) -> Result<String> {
        let time = UtcDateTime::now().format(&TIME_FORMAT).unwrap();
        let locale = ctx
            .user
            .preference
            .locale
            .as_ref()
            .map(|x| x.as_str())
            .unwrap_or("en-US");

        let rendering_ctx = DeepRenderingContext {
            time,
            locale,
            max_step_num: 7,
            user_prompt: None,
            plan_title: Some(plan_title),
            completed_steps: Some(completed_steps),
            current_step_title: Some(current_step_title),
            current_step_description: Some(current_step_description),
        };

        let template = self.env.get_template("deep_researcher")?;
        Ok(template.render(rendering_ctx)?)
    }

    pub fn render_reporter(&self, ctx: &CompletionContext) -> Result<String> {
        self.render_basic(DeepPromptKind::Reporter, ctx)
    }

    fn render_basic(&self, kind: DeepPromptKind, ctx: &CompletionContext) -> Result<String> {
        let time = UtcDateTime::now().format(&TIME_FORMAT).unwrap();
        let locale = ctx
            .user
            .preference
            .locale
            .as_ref()
            .map(|x| x.as_str())
            .unwrap_or("en-US");

        let rendering_ctx = DeepRenderingContext {
            time,
            locale,
            max_step_num: 7,
            user_prompt: ctx.latest_user_message(),
            plan_title: None,
            completed_steps: None,
            current_step_title: None,
            current_step_description: None,
        };

        let template_name = kind.as_str();
        let template = self.env.get_template(template_name)?;
        Ok(template.render(rendering_ctx)?)
    }
}
