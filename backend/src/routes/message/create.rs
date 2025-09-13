use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{Extension, Json, extract::State};
use entity::{MessageKind, ModelConfig, chunk, message, patch::ChunkKind, prelude::*};
use migration::{Expr, Mode};
use sea_orm::{EntityOrSelect, QueryOrder, prelude::*};
use serde::{Deserialize, Serialize};
use tokio::{select, task::yield_now};
use typeshare::typeshare;

use crate::{
    AppState,
    errors::*,
    middlewares::auth::UserId,
    openrouter::{self, StreamCompletionResp},
    prompts::{self, PromptStore},
    sse::{AssistantMessage, BufferChunk, EndKind, Publisher},
    tools::{self, ToolBox},
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageCreateReq {
    pub chat_id: i32,
    pub mode: MessageCreateReqMode,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum MessageCreateReqMode {
    Normal,
    Search,
    Agent,
    Research,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessageCreateResp {
    pub id: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MessageCreateReq>,
) -> JsonResult<MessageCreateResp> {
    let (user, chat_with_model, msgs_with_chunks) = tokio::join!(
        User::find_by_id(user_id).one(&app.conn),
        Chat::find_by_id(req.chat_id)
            .find_also_related(Model)
            .one(&app.conn),
        Message::find()
            .select()
            .filter(Expr::col(message::Column::ChatId).eq(req.chat_id))
            .order_by_asc(message::Column::Id)
            .find_with_related(Chunk)
            .all(&app.conn)
    );

    let (chat, model) = chat_with_model
        .kind(ErrorKind::Internal)?
        .into_iter()
        .next()
        .context("The request chat is not exists")
        .kind(ErrorKind::ResourceNotFound)?;

    if chat.owner_id != user_id {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }));
    }

    let model = model
        .context("The request model is not exists")
        .kind(ErrorKind::ResourceNotFound)?;

    let user = user
        .kind(ErrorKind::Internal)?
        .context("The request user is not exists")
        .kind(ErrorKind::ResourceNotFound)?;

    let msgs_with_chunks = msgs_with_chunks
        .context("Failed to load chat history")
        .kind(ErrorKind::Internal)?;
    let messages = get_message(msgs_with_chunks).kind(ErrorKind::Internal)?;

    let model_config = model
        .get_config()
        .context("Invaild model config")
        .kind(ErrorKind::Internal)?;

    let puber = app
        .sse
        .publish(req.chat_id)
        .await
        .kind(ErrorKind::Internal)?;
    let msg_id = puber
        .user_message(req.text)
        .await
        .kind(ErrorKind::Internal)?;

    let tool_set = match req.mode {
        MessageCreateReqMode::Normal => tools::NORMAL,
        MessageCreateReqMode::Search => tools::SEARCH,
        MessageCreateReqMode::Agent => tools::AGENT,
        MessageCreateReqMode::Research => tools::RESEARCH,
    };
    let (tool_prompts, tools) = app.tools.list(tool_set);
    let mut tool_box = app
        .tools
        .grab(req.chat_id, tool_set)
        .await
        .kind(ErrorKind::Internal)?;

    let system_prompt = prompts::ChatStore
        .template(user.preference.locale.as_deref())
        .await
        .kind(ErrorKind::Internal)?
        .render(&app.prompt, req.chat_id, tool_prompts, (), ())
        .await
        .kind(ErrorKind::Internal)?;

    puber.spawn_scope(|puber| async move {
        let assistant = puber
            .new_assistant_message()
            .await
            .raw_kind(ErrorKind::Internal)?;
        let mut buffer_chunk = None;

        let res = handle_sse(
            app.clone(),
            &assistant,
            &mut buffer_chunk,
            model_config,
            system_prompt,
            tools,
            &mut tool_box,
            &puber,
            messages,
        )
        .await;
        let kind = match res {
            Ok(kind) => kind,
            Err(err) => {
                puber.raw_token(Err(err));

                EndKind::Error
            }
        };
        if let Some(bc) = buffer_chunk {
            bc.end_buffer_chunk(kind)
                .await
                .raw_kind(ErrorKind::Internal)?;
        }
        assistant
            .end_message(kind)
            .await
            .raw_kind(ErrorKind::Internal)?;

        app.tools
            .put_back(tool_box)
            .await
            .raw_kind(ErrorKind::Internal)?;
        Ok(())
    });

    Ok(Json(MessageCreateResp { id: msg_id }))
}

async fn handle_sse<'a>(
    app: Arc<AppState>,
    assistant: &'a AssistantMessage<'a>,
    buffer_chunk: &mut Option<BufferChunk<'a, 'a>>,
    model: ModelConfig,
    system_prompt: String,
    tools: Vec<openrouter::Tool>,
    tool_box: &mut ToolBox,
    puber: &Publisher,
    messages: Vec<openrouter::Message>,
) -> Result<EndKind, Error> {
    let mut tool_calls: Vec<openrouter::MessageToolCall> = vec![];

    let mut systemed_messages = vec![openrouter::Message::System(system_prompt)];
    systemed_messages.extend(messages);

    loop {
        for tool_call in tool_calls.drain(..) {
            let Some((name, tool)) = tool_box.get(&tool_call.name.as_str()) else {
                continue;
            };

            assistant.start_tool_call(name, tool_call.arguments.clone());
            let output = tool
                .call(&tool_call.arguments)
                .await
                .raw_kind(ErrorKind::ToolCallFail);
            let content =
                serde_json::to_string(&JsonUnion::from(output)).raw_kind(ErrorKind::Internal)?;
            assistant
                .end_tool_call(name, tool_call.arguments, content, tool_call.id)
                .await
                .raw_kind(ErrorKind::Internal)?;
        }

        let mut completion = app
            .openrouter
            .stream(
                systemed_messages.clone(),
                model.clone().into(),
                tools.clone(),
            )
            .await
            .raw_kind(ErrorKind::ApiFail)?;

        loop {
            select! {
                biased;
                _ = puber.on_halt() => {
                    return Ok(EndKind::Halt);
                }

                token = completion.next() => {
                    match token {
                        Some(Ok(token)) => match token {
                            StreamCompletionResp::ReasoningToken(token) => {
                                if token.is_empty() {
                                    continue;
                                }

                                match buffer_chunk.take_if(|bc| bc.kind() != ChunkKind::Reasoning) {
                                    Some(bc) => {
                                        bc.end_buffer_chunk(EndKind::Complete)
                                            .await
                                            .raw_kind(ErrorKind::Internal)?;
                                        yield_now().await;
                                        *buffer_chunk =
                                            Some(assistant.new_buffer_chunk(ChunkKind::Reasoning).await);
                                    }
                                    None if buffer_chunk.is_none() => {
                                        *buffer_chunk =
                                            Some(assistant.new_buffer_chunk(ChunkKind::Reasoning).await);
                                    }
                                    _ => {}
                                }
                                buffer_chunk
                                    .as_ref()
                                    .unwrap()
                                    .send_token(&token)
                                    .await
                                    .raw_kind(ErrorKind::Internal)?;
                            }
                            StreamCompletionResp::ResponseToken(token) => {
                                if token.is_empty() {
                                    continue;
                                }

                                match buffer_chunk.take_if(|bc|bc.kind() != ChunkKind::Text) {
                                    Some(bc) => {
                                        bc.end_buffer_chunk(EndKind::Complete)
                                            .await
                                            .raw_kind(ErrorKind::Internal)?;
                                        yield_now().await;
                                        *buffer_chunk = Some(assistant.new_buffer_chunk(ChunkKind::Text).await);
                                    }
                                    None if buffer_chunk.is_none() => {
                                        *buffer_chunk = Some(assistant.new_buffer_chunk(ChunkKind::Text).await);
                                    }
                                    _ => {}
                                }
                                buffer_chunk
                                    .as_ref()
                                    .unwrap()
                                    .send_token(&token)
                                    .await
                                    .raw_kind(ErrorKind::Internal)?;
                            }
                            StreamCompletionResp::ToolCall { name, args, id } => {
                                tool_calls.push(openrouter::MessageToolCall {
                                    id,
                                    name,
                                    arguments: args,
                                })
                            }
                            _ => {}
                        },
                        Some(Err(err)) => {
                            return Err(Error {
                                error: ErrorKind::ApiFail,
                                reason: err.to_string(),
                            });
                        }
                        None => break,
                    }
                }
            };
        }
        if let Some(bc) = buffer_chunk.take() {
            bc.end_buffer_chunk(EndKind::Complete)
                .await
                .raw_kind(ErrorKind::Internal)?;
        }
        if tool_calls.is_empty() {
            break;
        }
    }

    Ok(EndKind::Complete)
}

fn get_message(
    messages_with_chunks: Vec<(message::Model, Vec<chunk::Model>)>,
) -> Result<Vec<openrouter::Message>> {
    let mut messages = vec![];
    for (message, chunks) in messages_with_chunks {
        match message.kind {
            MessageKind::Hidden => continue,
            MessageKind::User => messages.extend(
                chunks
                    .into_iter()
                    .map(|chunk| openrouter::Message::User(chunk.content)),
            ),
            MessageKind::Assistant => {
                for chunk in chunks {
                    match chunk.kind {
                        ChunkKind::Text => {
                            messages.push(openrouter::Message::Assistant(chunk.content))
                        }
                        ChunkKind::Reasoning => continue,
                        ChunkKind::ToolCall => {
                            let tool_call = chunk.as_tool_call()?;

                            messages.extend([
                                openrouter::Message::ToolCall(openrouter::MessageToolCall {
                                    id: tool_call.id.clone(),
                                    name: tool_call.name,
                                    arguments: tool_call.args,
                                }),
                                openrouter::Message::ToolResult(openrouter::MessageToolResult {
                                    id: tool_call.id,
                                    content: tool_call.content,
                                }),
                            ]);
                        }
                    }
                }
            }
        }
    }

    Ok(messages)
}
