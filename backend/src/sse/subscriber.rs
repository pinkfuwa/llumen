use std::{collections::hash_map::Entry, sync::Arc};

use anyhow::Result;
use futures_util::{
    Stream, StreamExt,
    stream::{self, BoxStream},
};
use tokio::sync::{Notify, RwLock};
use tokio::{select, sync::broadcast};

use crate::{
    errors::*,
    sse::{SseContext, Token},
};

use super::context::SseInner;

pub struct Subscriber {
    st: BoxStream<'static, Result<Token, Error>>,
}

struct State {
    inner: Arc<RwLock<SseInner>>,
    on_receive: Arc<Notify>,
    channel: broadcast::Receiver<Result<Token, Error>>,
    offset: usize,
}
impl Stream for Subscriber {
    type Item = Result<Token, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.st.poll_next_unpin(cx)
    }
}

impl Subscriber {
    pub(super) async fn new(ctx: &SseContext, chat_id: i32) -> Result<Self> {
        let (state, st) = match ctx.map.lock().await.entry(chat_id) {
            Entry::Occupied(entry) => {
                let inner = entry.get().read().await;

                let st = stream::iter([Ok(Token::Last(inner.last_id, inner.version))]);

                let state = State {
                    inner: entry.get().clone(),
                    on_receive: inner.on_receive.clone(),
                    channel: inner.channel.subscribe(),
                    offset: 0,
                };
                (state, st)
            }
            Entry::Vacant(entry) => {
                let inner = SseInner::new(ctx).await?;
                let on_receive = inner.on_receive.clone();
                let channel = inner.channel.subscribe();

                let st = stream::iter([Ok(Token::Last(inner.last_id, inner.version))]);
                let inner = entry.insert(Arc::new(RwLock::new(inner))).clone();

                let state = State {
                    inner,
                    on_receive,
                    channel,
                    offset: 0,
                };
                (state, st)
            }
        };

        let st = st
            .chain(stream::unfold(state, |mut state| async move {
                let res = select! {
                    biased;

                    _ = state.on_receive.notified() => {
                        handle_buffer(&mut state).await
                    }

                    res = state.channel.recv() => {
                        handle_channel(&mut state, res).await
                    }
                };

                Some((res, state))
            }))
            .boxed();

        Ok(Subscriber { st })
    }
}

async fn handle_channel(
    state: &mut State,
    res: Result<Result<Token, Error>, impl ToString>,
) -> Result<Token, Error> {
    let token = res.map_err(|e| Error {
        error: ErrorKind::Internal,
        reason: e.to_string(),
    })??;

    match token {
        Token::End(_) => {
            state.offset = 0;
        }
        // don't care token
        _ => {}
    }
    Ok(token)
}

async fn handle_buffer(state: &mut State) -> Result<Token, Error> {
    let inner = state.inner.read().await;

    let token = inner.buffer[state.offset..].to_owned();
    state.offset = inner.buffer.len();

    drop(inner);
    Ok(Token::Token(token))
}
