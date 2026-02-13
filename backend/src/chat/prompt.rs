//! Prompt rendering via minijinja + rust_embed_for_web.
//!
//! Templates live in `agent/prompt/` and are embedded at compile time.
//! Each render method populates locale/time/model variables and returns
//! the final system-prompt string.

use anyhow::{Context as _, Result};
use minijinja::Environment;
use rust_embed_for_web::{EmbedableFile, RustEmbed};
use time::macros::format_description;

#[derive(RustEmbed)]
#[folder = "../agent/prompt"]
#[gzip = false]
struct PromptAssets;

fn load_template(name: &str) -> Result<String> {
    let file =
        PromptAssets::get(name).with_context(|| format!("prompt template not found: {name}"))?;
    let bytes = file.data();
    Ok(std::str::from_utf8(bytes.as_ref())?.to_string())
}

pub(crate) const TIME_FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
    format_description!("[weekday], [hour]:[minute], [day] [month repr:long] [year]");

fn current_time() -> String {
    time::OffsetDateTime::now_utc()
        .format(TIME_FORMAT)
        .unwrap_or_default()
}

pub struct Prompt {
    env: Environment<'static>,
}

impl Prompt {
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();

        let templates = [
            ("normal", "normal.j2"),
            ("search", "search.j2"),
            ("coordinator", "coordinator.j2"),
            ("context", "context.j2"),
            ("title_generation", "title_generation.j2"),
            ("deep/prompt_enhancer", "deepresearch/prompt_enhancer.j2"),
            ("deep/planner", "deepresearch/planner.j2"),
            ("deep/researcher", "deepresearch/researcher.j2"),
            ("deep/reporter", "deepresearch/reporter.j2"),
            ("deep/report_input", "deepresearch/report_input.j2"),
            ("deep/step_input", "deepresearch/step_input.j2"),
            (
                "deep/step_system_message",
                "deepresearch/step_system_message.j2",
            ),
            ("deep/coder", "deepresearch/coder.j2"),
            ("deep/coordinator", "deepresearch/coordinator.j2"),
        ];

        for (name, path) in templates {
            let src = load_template(path)?;
            // Leak the template source to get 'static lifetime — these are
            // compile-time-embedded templates that live for the entire program.
            let src: &'static str = Box::leak(src.into_boxed_str());
            env.add_template(name, src)
                .with_context(|| format!("failed to parse template: {path}"))?;
        }

        Ok(Self { env })
    }

    // ------------------------------------------------------------------
    // Normal & Search modes
    // ------------------------------------------------------------------

    pub fn render_normal(
        &self,
        locale: &str,
        model_name: &str,
        model_provider: &str,
    ) -> Result<String> {
        let tmpl = self.env.get_template("normal")?;
        Ok(tmpl.render(minijinja::context! {
            locale,
            model_name,
            model_provider,
        })?)
    }

    pub fn render_search(
        &self,
        locale: &str,
        model_name: &str,
        model_provider: &str,
    ) -> Result<String> {
        let tmpl = self.env.get_template("search")?;
        Ok(tmpl.render(minijinja::context! {
            locale,
            model_name,
            model_provider,
        })?)
    }

    pub fn render_context(
        &self,
        llumen_related: bool,
        time: &str,
        chat_title: Option<&str>,
    ) -> Result<String> {
        let tmpl = self.env.get_template("context")?;
        Ok(tmpl.render(minijinja::context! {
            llumen_related,
            time,
            chat_title,
        })?)
    }

    pub fn render_title_generation(&self, locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("title_generation")?;
        Ok(tmpl.render(minijinja::context! { locale })?)
    }

    // ------------------------------------------------------------------
    // Deep Research mode
    // ------------------------------------------------------------------

    pub fn render_coordinator(&self, _locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("deep/coordinator")?;
        Ok(tmpl.render(minijinja::context! { time => current_time() })?)
    }

    pub fn render_prompt_enhancer(&self, _locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("deep/prompt_enhancer")?;
        Ok(tmpl.render(minijinja::context! { time => current_time() })?)
    }

    pub fn render_planner(&self, locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("deep/planner")?;
        Ok(tmpl.render(minijinja::context! {
            time => current_time(),
            locale,
            max_step_num => 5,
        })?)
    }

    pub fn render_researcher(&self, locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("deep/researcher")?;
        Ok(tmpl.render(minijinja::context! {
            time => current_time(),
            locale,
        })?)
    }

    pub fn render_coder(&self, locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("deep/coder")?;
        Ok(tmpl.render(minijinja::context! {
            time => current_time(),
            locale,
        })?)
    }

    pub fn render_reporter(&self, locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("deep/reporter")?;
        Ok(tmpl.render(minijinja::context! {
            time => current_time(),
            locale,
        })?)
    }

    pub fn render_step_input(&self, ctx: &StepInputContext<'_>) -> Result<String> {
        let tmpl = self.env.get_template("deep/step_input")?;
        Ok(tmpl.render(minijinja::context! {
            plan_title => ctx.plan_title,
            completed_steps => ctx.completed_steps.iter().map(|s| {
                minijinja::context! { title => s.title, content => s.content }
            }).collect::<Vec<_>>(),
            current_step_title => ctx.current_step_title,
            current_step_description => ctx.current_step_description,
            locale => ctx.locale,
        })?)
    }

    pub fn render_step_system_message(&self, _locale: &str) -> Result<String> {
        let tmpl = self.env.get_template("deep/step_system_message")?;
        Ok(tmpl.render(minijinja::context! {})?)
    }

    pub fn render_report_input(&self, ctx: &ReportInputContext<'_>) -> Result<String> {
        let tmpl = self.env.get_template("deep/report_input")?;
        Ok(tmpl.render(minijinja::context! {
            plan_title => ctx.plan_title,
            enhanced_prompt => ctx.enhanced_prompt,
            completed_steps => ctx.completed_steps.iter().map(|s| {
                minijinja::context! { title => s.title, content => s.content }
            }).collect::<Vec<_>>(),
        })?)
    }
}

// ------------------------------------------------------------------
// Context types for deep-research template rendering
// ------------------------------------------------------------------

pub struct CompletedStep<'a> {
    pub title: &'a str,
    pub content: &'a str,
}

pub struct StepInputContext<'a> {
    pub locale: &'a str,
    pub plan_title: &'a str,
    pub completed_steps: Vec<CompletedStep<'a>>,
    pub current_step_title: &'a str,
    pub current_step_description: &'a str,
}

#[allow(dead_code)]
pub struct ReportInputContext<'a> {
    pub locale: &'a str,
    pub plan_title: &'a str,
    pub enhanced_prompt: &'a str,
    pub completed_steps: Vec<CompletedStep<'a>>,
}
