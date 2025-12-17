use minijinja::Environment;
use protocol::ModelConfig;
use serde::Serialize;
use time::UtcDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::utils::model::ModelChecker;

use super::context::CompletionContext;

#[derive(Debug, Clone, Copy)]
pub enum PromptKind {
    Normal,
    Search,
    TitleGen,
    Coordinator,
}

impl PromptKind {
    fn as_str(&self) -> &'static str {
        match self {
            PromptKind::Normal => "normal",
            PromptKind::Search => "search",
            PromptKind::TitleGen => "title",
            PromptKind::Coordinator => "coordinator",
        }
    }
}

pub struct Prompt {
    env: Environment<'static>,
}

impl Prompt {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.add_template(
            "title",
            include_str!("../../../prompts/title_generation.j2"),
        )
        .unwrap();
        env.add_template("normal", include_str!("../../../prompts/normal.j2"))
            .unwrap();
        env.add_template("search", include_str!("../../../prompts/search.j2"))
            .unwrap();
        env.add_template(
            "coordinator",
            include_str!("../../../prompts/coordinator.j2"),
        )
        .unwrap();
        env.add_global("repo_url", "https://github.com/pinkfuwa/llumen");
        env.add_global("repo_readme", include_str!("../../../README.md"));
        Self { env }
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
}

const TIME_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[weekday], [year]-[month]-[day]");

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

        let rendering_ctx = RenderingContext {
            model: config,
            user_id: ctx.user.id,
            username: &ctx.user.name,
            chat_id: ctx.chat.id.clone().unwrap(),
            chat_title,
            locale: ctx.user.preference.locale.as_ref().map(|x| x.as_str()),
            time,
            user_prompt: ctx.latest_user_message(),
        };

        let template_name = kind.as_str();
        let template = self.env.get_template(template_name)?;
        template.render(rendering_ctx)
    }
}
