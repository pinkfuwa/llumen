use anyhow::Result;
use minijinja::Environment;
use serde::Serialize;
use time::UtcDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::chat::CompletionContext;

#[derive(Serialize, Clone)]
pub struct CompletedStep {
    pub title: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct PromptContext {
    pub time: String,
    pub locale: String,
    pub max_step_num: usize,
    pub user_prompt: Option<String>,
    // For researcher and coder: completed steps and current step
    pub plan_title: Option<String>,
    pub completed_steps: Option<Vec<CompletedStep>>,
    pub current_step_title: Option<String>,
    pub current_step_description: Option<String>,
    pub enhanced_prompt: Option<String>,
}

pub struct DeepPrompt {
    env: Environment<'static>,
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
        env.add_template(
            "step_system_message",
            include_str!("../../../prompts/deepresearch/step_system_message.md"),
        )
        .unwrap();
        env.add_template(
            "step_input",
            include_str!("../../../prompts/deepresearch/step_input.md"),
        )
        .unwrap();
        env.add_template(
            "report_input",
            include_str!("../../../prompts/deepresearch/report_input.md"),
        )
        .unwrap();
        Self { env }
    }

    pub fn render_coordinator(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("deep_coordinator")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_prompt_enhancer(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("deep_prompt_enhancer")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_planner(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("deep_planner")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_researcher(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("deep_researcher")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_coder(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("deep_coder")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_reporter(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("deep_reporter")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_step_system_message(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("step_system_message")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_step_input(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("step_input")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_report_input(&self, ctx: &PromptContext) -> Result<String> {
        let template = self.env.get_template("report_input")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }
}
