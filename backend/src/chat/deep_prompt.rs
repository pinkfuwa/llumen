use anyhow::Result;
use minijinja::Environment;
use serde::Serialize;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

const TIME_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[weekday], [hour]:[minute], [day] [month] [year]");

#[derive(Serialize, Clone)]
pub struct CompletedStep<'a> {
    pub title: &'a str,
    pub content: &'a str,
}

#[derive(Serialize)]
pub struct StepInputContext<'a> {
    pub locale: &'a str,
    pub plan_title: &'a str,
    pub completed_steps: Vec<CompletedStep<'a>>,
    pub current_step_title: &'a str,
    pub current_step_description: &'a str,
}

#[derive(Serialize)]
pub struct ReportInputContext<'a> {
    pub locale: &'a str,
    pub plan_title: &'a str,
    pub completed_steps: Vec<CompletedStep<'a>>,
    pub enhanced_prompt: &'a str,
}

#[derive(Serialize)]
struct BasicContext {
    pub time: String,
    pub locale: String,
    pub max_step_num: usize,
}

impl BasicContext {
    pub fn new(locale: String) -> Self {
        let time = time::OffsetDateTime::now_utc().format(TIME_FORMAT).unwrap();
        BasicContext {
            time,
            locale,
            max_step_num: 8,
        }
    }
}

pub struct DeepPrompt {
    env: Environment<'static>,
}

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

    pub fn render_coordinator(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template("deep_coordinator")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_prompt_enhancer(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template("deep_prompt_enhancer")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_planner(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template("deep_planner")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_researcher(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template("deep_researcher")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_coder(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template("deep_coder")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_reporter(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template("deep_reporter")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_step_system_message(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template("step_system_message")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_step_input(&self, ctx: &StepInputContext) -> Result<String> {
        let template = self.env.get_template("step_input")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_report_input(&self, ctx: &ReportInputContext) -> Result<String> {
        let template = self.env.get_template("report_input")?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }
}
