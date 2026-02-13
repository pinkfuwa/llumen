use std::sync::Arc;

use anyhow::Context as _;
use sea_orm::DatabaseConnection;

use super::prompt::Prompt;
use super::session::CompletionSession;
use super::strategies::{self, Strategy};
use super::token::Token;
use super::tools::Tools;

pub(crate) use super::channel;

#[derive(Debug, Clone, Copy)]
pub enum StreamEndReason {
    Halt,
    Exhausted,
}

/// The global context for the chat system.
pub struct Context {
    pub(crate) db: DatabaseConnection,
    pub(crate) openrouter: Arc<crate::openrouter::Openrouter>,
    pub(crate) channel: Arc<channel::Context<Token>>,
    pub(crate) prompt: Arc<Prompt>,
    pub(crate) blob: Arc<crate::utils::blob::BlobDB>,
    pub(crate) tools: Tools,
}

impl Context {
    /// Creates a chat subsystem context that prepares routing, prompt, and tool
    /// dependencies.
    pub fn new(
        db: DatabaseConnection,
        openrouter: Arc<crate::openrouter::Openrouter>,
        blob: Arc<crate::utils::blob::BlobDB>,
    ) -> Result<Self, anyhow::Error> {
        let prompt = Prompt::new().context("failed to load prompt templates")?;

        Ok(Self {
            db,
            openrouter,
            channel: Arc::new(channel::Context::new()),
            prompt: Arc::new(prompt),
            blob,
            tools: Tools::new(),
        })
    }

    /// Prepares a chat session for the specified user/chat/model tuple.
    pub fn get_session(
        self: &Arc<Self>,
        user_id: i32,
        chat_id: i32,
        model_id: i32,
    ) -> impl std::future::Future<Output = Result<CompletionSession, anyhow::Error>> + '_ {
        CompletionSession::new(self.clone(), user_id, chat_id, model_id)
    }

    /// Halts the active session immediately, notifying any subscribers.
    pub async fn halt_session(&self, chat_id: i32) {
        self.channel.stop(chat_id).await
    }

    /// Subscribes to a session, optionally resuming from a cursor.
    pub fn subscribe(
        self: Arc<Self>,
        chat_id: i32,
        cursor: Option<channel::Cursor>,
    ) -> impl tokio_stream::Stream<Item = Token> + 'static {
        self.channel.clone().subscribe(chat_id, cursor)
    }

    /// Returns true when no publisher currently owns the channel for the chat.
    pub fn is_streaming(&self, chat_id: i32) -> bool {
        !self.channel.publishable(chat_id)
    }

    /// Runs a complete chat turn: dispatches the strategy, then saves.
    /// Called from a spawned task.
    pub async fn process(
        self: Arc<Self>,
        strategy: Strategy,
        mut session: CompletionSession,
    ) -> anyhow::Result<()> {
        // Emit Start token
        let user_msg_id = session.history.last().map(|m| m.id).unwrap_or(0);
        session.add_token(Token::Start {
            id: session.message.id,
            user_msg_id,
        });

        // Run the selected strategy
        if let Err(e) = strategies::dispatch(self.clone(), strategy, &mut session).await {
            log::error!("completion error: {e:#}");
            session.add_error(format!("{e}"));
        }

        // Generate title if this is a new chat
        if let Err(e) = session.try_generate_title().await {
            log::error!("title generation error: {e:#}");
        }

        // Persist
        session.save().await?;

        Ok(())
    }
}
