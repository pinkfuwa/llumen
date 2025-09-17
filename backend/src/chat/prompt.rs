use minijinja::Environment;
use serde::Serialize;

use super::context::CompletionContext;

#[derive(Debug, Clone, Copy)]
pub enum PromptKind {
    Normal,
    Search,
    TitleGen,
}

impl PromptKind {
    fn as_str(&self) -> &'static str {
        match self {
            PromptKind::Normal => "normal",
            PromptKind::Search => "search",
            PromptKind::TitleGen => "title",
        }
    }
}

pub struct Prompt {
    env: Environment<'static>,
}

impl Prompt {
    pub fn new() -> Self {
        let mut env = Environment::new();
        // TODO: Add prompt template
        Self { env }
    }
}

#[derive(Serialize)]
struct RenderingContext<'a> {
    model: entity::ModelConfig,
    user_id: i32,
    user_name: &'a str,
    chat_id: i32,
    chat_title: Option<&'a str>,
    locale: Option<&'a str>,
}

impl Prompt {
    pub fn render(
        &self,
        kind: PromptKind,
        ctx: &CompletionContext,
    ) -> Result<String, minijinja::Error> {
        let config = ctx.model.get_config().unwrap();

        let chat_title = ctx.chat.title.try_as_ref();
        let chat_title = chat_title.and_then(|x| x.as_deref());

        let rendering_ctx = RenderingContext {
            model: config,
            user_id: ctx.user.id,
            user_name: &ctx.user.name,
            chat_id: ctx.chat.id.clone().unwrap(),
            chat_title,
            locale: ctx.user.preference.locale.as_ref().map(|x| x.as_str()),
        };

        let template_name = kind.as_str();
        let template = self.env.get_template(template_name)?;
        template.render(rendering_ctx)
    }
}
