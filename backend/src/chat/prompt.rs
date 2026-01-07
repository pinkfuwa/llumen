use anyhow::Result;
use minijinja::Environment;
use protocol::ModelConfig;
use rust_embed_for_web::{EmbedableFile, RustEmbed};
use serde::Serialize;
use time::UtcDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::utils::model::ModelChecker;

use super::context::CompletionContext;

#[derive(RustEmbed)]
#[folder = "../agent/prompt"]
#[br = false]
#[gzip = false]
struct PromptAssets;

#[derive(Debug, Clone, Copy)]
pub enum PromptKind {
    Normal,
    Search,
    TitleGen,
    Coordinator,
    Context,
}

impl PromptKind {
    fn as_str(&self) -> &'static str {
        match self {
            PromptKind::Normal => "normal",
            PromptKind::Search => "search",
            PromptKind::TitleGen => "title",
            PromptKind::Coordinator => "coordinator",
            PromptKind::Context => "context",
        }
    }

    fn file_name(&self) -> &'static str {
        match self {
            PromptKind::Normal => "normal.j2",
            PromptKind::Search => "search.j2",
            PromptKind::TitleGen => "title_generation.j2",
            PromptKind::Coordinator => "coordinator.j2",
            PromptKind::Context => "context.j2",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeepPromptKind {
    Coordinator,
    PromptEnhancer,
    Planner,
    Researcher,
    Coder,
    Reporter,
    StepSystemMessage,
    StepInput,
    ReportInput,
}

impl DeepPromptKind {
    fn as_str(&self) -> &'static str {
        match self {
            DeepPromptKind::Coordinator => "deep_coordinator",
            DeepPromptKind::PromptEnhancer => "deep_prompt_enhancer",
            DeepPromptKind::Planner => "deep_planner",
            DeepPromptKind::Researcher => "deep_researcher",
            DeepPromptKind::Coder => "deep_coder",
            DeepPromptKind::Reporter => "deep_reporter",
            DeepPromptKind::StepSystemMessage => "step_system_message",
            DeepPromptKind::StepInput => "step_input",
            DeepPromptKind::ReportInput => "report_input",
        }
    }

    fn file_name(&self) -> &'static str {
        match self {
            DeepPromptKind::Coordinator => "deepresearch/coordinator.j2",
            DeepPromptKind::PromptEnhancer => "deepresearch/prompt_enhancer.j2",
            DeepPromptKind::Planner => "deepresearch/planner.j2",
            DeepPromptKind::Researcher => "deepresearch/researcher.j2",
            DeepPromptKind::Coder => "deepresearch/coder.j2",
            DeepPromptKind::Reporter => "deepresearch/reporter.j2",
            DeepPromptKind::StepSystemMessage => "deepresearch/step_system_message.j2",
            DeepPromptKind::StepInput => "deepresearch/step_input.j2",
            DeepPromptKind::ReportInput => "deepresearch/report_input.j2",
        }
    }
}

pub struct Prompt {
    env: Environment<'static>,
    _templates: Vec<String>,
}

impl Prompt {
    pub fn new() -> Result<Self> {
        let mut templates = Vec::new();
        let mut env = Environment::new();

        // Load normal prompts
        for kind in [
            PromptKind::Normal,
            PromptKind::Search,
            PromptKind::TitleGen,
            PromptKind::Coordinator,
            PromptKind::Context,
        ] {
            let content = PromptAssets::get(kind.file_name())
                .ok_or_else(|| anyhow::anyhow!("Prompt file not found: {}", kind.file_name()))?;
            let template_str = std::str::from_utf8(content.data().as_ref())?.to_string();
            templates.push(template_str);
        }

        // Load deep research prompts
        for kind in [
            DeepPromptKind::Coordinator,
            DeepPromptKind::PromptEnhancer,
            DeepPromptKind::Planner,
            DeepPromptKind::Researcher,
            DeepPromptKind::Coder,
            DeepPromptKind::Reporter,
            DeepPromptKind::StepSystemMessage,
            DeepPromptKind::StepInput,
            DeepPromptKind::ReportInput,
        ] {
            let content = PromptAssets::get(kind.file_name())
                .ok_or_else(|| anyhow::anyhow!("Prompt file not found: {}", kind.file_name()))?;
            let template_str = std::str::from_utf8(content.data().as_ref())?.to_string();
            templates.push(template_str);
        }

        // Add templates to environment using leaked static references
        let template_refs: Vec<&'static str> = templates
            .iter()
            .map(|s| {
                let leaked: &'static str = Box::leak(s.clone().into_boxed_str());
                leaked
            })
            .collect();

        let kinds = [
            PromptKind::Normal.as_str(),
            PromptKind::Search.as_str(),
            PromptKind::TitleGen.as_str(),
            PromptKind::Coordinator.as_str(),
            PromptKind::Context.as_str(),
            DeepPromptKind::Coordinator.as_str(),
            DeepPromptKind::PromptEnhancer.as_str(),
            DeepPromptKind::Planner.as_str(),
            DeepPromptKind::Researcher.as_str(),
            DeepPromptKind::Coder.as_str(),
            DeepPromptKind::Reporter.as_str(),
            DeepPromptKind::StepSystemMessage.as_str(),
            DeepPromptKind::StepInput.as_str(),
            DeepPromptKind::ReportInput.as_str(),
        ];

        for (name, template) in kinds.iter().zip(template_refs.iter()) {
            env.add_template(name, template)?;
        }

        Ok(Self {
            env,
            _templates: templates,
        })
    }
}

#[derive(Serialize)]
struct RenderingContext<'a> {
    model: protocol::ModelConfig,
    user_id: i32,
    username: &'a str,
    chat_id: i32,
    chat_title: Option<&'a str>,
    locale: Option<&'a str>,
    time: String,
    user_prompt: Option<&'a str>,
    model_name: &'a str,
    model_provider: &'a str,
}

#[derive(Serialize)]
struct ContextRenderingContext<'a> {
    chat_title: Option<&'a str>,
    time: String,
    llumen_related: bool,
}

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
        let time = time::OffsetDateTime::now_utc()
            .format(&DEEP_TIME_FORMAT)
            .unwrap();
        BasicContext {
            time,
            locale,
            max_step_num: 8,
        }
    }
}

const TIME_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[weekday], [year]-[month repr:long]-[day]");

const DEEP_TIME_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[weekday], [hour]:[minute]:[second], [day] [month repr:long] [year]");

const CONTEXT_TIME_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[weekday], [hour]:[minute], [day] [month repr:long] [year]");

impl Prompt {
    pub fn render(
        &self,
        kind: PromptKind,
        ctx: &CompletionContext,
    ) -> Result<String, minijinja::Error> {
        let config = <ModelConfig as ModelChecker>::from_toml(&ctx.model.config).unwrap();

        let chat_title = ctx.chat.title.try_as_ref();
        let chat_title = chat_title.and_then(|x| x.as_deref());

        let time = UtcDateTime::now().format(&TIME_FORMAT).unwrap();

        let model_id = ctx
            .get_model_config()
            .ok()
            .map(|x| x.model_id)
            .unwrap_or_default();
        let (model_name, model_provider) = {
            let split = model_id.char_indices().find(|(_, x)| *x == '/');
            match split {
                Some((index, _)) => (&model_id[..index], &model_id[index + 1..]),
                None => (model_id.as_str(), ""),
            }
        };

        let rendering_ctx = RenderingContext {
            model: config,
            user_id: ctx.user.id,
            username: &ctx.user.name,
            chat_id: ctx.chat.id.clone().unwrap(),
            chat_title,
            locale: ctx.user.preference.locale.as_ref().map(|x| x.as_str()),
            time,
            user_prompt: ctx.latest_user_message(),
            model_name,
            model_provider,
        };

        let template_name = kind.as_str();
        let template = self.env.get_template(template_name)?;
        template.render(rendering_ctx)
    }

    pub fn render_context(&self, ctx: &CompletionContext) -> Result<String, minijinja::Error> {
        let chat_title = ctx.chat.title.try_as_ref();
        let chat_title = chat_title.and_then(|x| x.as_deref());

        let time = UtcDateTime::now().format(&CONTEXT_TIME_FORMAT).unwrap();

        let user_prompt = ctx.latest_user_message().unwrap_or("");
        let llumen_related = user_prompt.to_lowercase().contains("llumen")
            || user_prompt.contains("流明")
            || user_prompt.to_lowercase().contains("app")
            || user_prompt.to_lowercase().contains("you")
            || user_prompt.to_lowercase().contains("self-analysis");

        let rendering_ctx = ContextRenderingContext {
            chat_title,
            time,
            llumen_related,
        };

        let template = self.env.get_template(PromptKind::Context.as_str())?;
        template.render(rendering_ctx)
    }

    pub fn render_prompt_enhancer(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self
            .env
            .get_template(DeepPromptKind::PromptEnhancer.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_planner(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template(DeepPromptKind::Planner.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_researcher(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template(DeepPromptKind::Researcher.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_coder(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template(DeepPromptKind::Coder.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_reporter(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self.env.get_template(DeepPromptKind::Reporter.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_step_system_message(&self, locale: &str) -> Result<String> {
        let ctx = BasicContext::new(locale.to_string());
        let template = self
            .env
            .get_template(DeepPromptKind::StepSystemMessage.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_step_input(&self, ctx: &StepInputContext) -> Result<String> {
        let template = self.env.get_template(DeepPromptKind::StepInput.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }

    pub fn render_report_input(&self, ctx: &ReportInputContext) -> Result<String> {
        let template = self
            .env
            .get_template(DeepPromptKind::ReportInput.as_str())?;
        let rendered = template.render(&ctx)?;
        Ok(rendered)
    }
}
