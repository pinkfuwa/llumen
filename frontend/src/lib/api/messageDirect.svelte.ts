import { events } from 'fetch-event-stream';

import { APIFetch, getError, RawAPIFetch } from './state/errorHandle';

import { CreateMutation, CreateRawMutation } from './state';
import type { MutationResult, RawMutationResult } from './state/mutate';
import type {
	MessageDeleteReq,
	MessageCreateReq,
	MessageCreateResp,
	MessagePaginateRespChunk,
	MessagePaginateResp,
	MessagePaginateReq,
	MessagePaginateRespList,
	SseReq,
	SseResp,
	MessagePaginateRespChunkKindFile,
	MessagePaginateRespChunkKind
} from './types';
import { MessagePaginateReqOrder, MessagePaginateRespRole } from './types';
import { dispatchError } from '$lib/error';
import { UpdateInfiniteQueryDataById } from './state';
import { untrack } from 'svelte';

let version = $state(-1);

let messages = $state<Array<MessagePaginateRespList & { stream?: boolean }>>([]);

let streaming = $derived(!!messages.at(-1)?.stream);

const Handlers: {
	[key in SseResp['t']]: (data: Extract<SseResp, { t: key }>['c'], chatId: number) => void;
} = {
	version: (data, chatId) => {
		if (version !== data.version) {
			version = data.version;
			syncMessages(chatId);
		}
	},

	start: (data) => {
		if (messages.at(-1)?.id == data.user_msg_id) console.warn('Duplicate message detected');
		messages.unshift({
			id: data.id,
			role: MessagePaginateRespRole.Assistant,
			chunks: [],
			token: 0,
			price: 0,
			stream: true
		});
	},

	token: (token) => {
		handleTokenChunk('text', { content: token.content });
	},

	reasoning: (reasoning) => {
		handleTokenChunk('reasoning', { content: reasoning.content });
	},

	tool_call: (toolCall) => {
		handleTokenChunk('tool_call', {
			name: toolCall.name,
			args: toolCall.args,
			content: ''
		});
	},

	tool_result: (toolResult) => {
		handleTokenChunk('tool_result', { content: toolResult.content });
	},

	complete: (data) => {
		const lastMsg = messages.at(-1);
		if (lastMsg && lastMsg.stream) {
			lastMsg.stream = false;
			lastMsg.token = data.token_count;
			lastMsg.price = data.cost;
		}
	},

	title: (data, chatId) => {
		UpdateInfiniteQueryDataById({
			key: ['chatPaginate'],
			id: chatId,
			updater: (chat) => ({ ...chat, title: data.title })
		});
	},

	error: (err) => {
		const lastMsg = messages.at(-1);
		if (lastMsg && lastMsg.stream) {
			lastMsg.chunks.push({
				id: Date.now(),
				kind: { t: 'error', c: { content: err.content } }
			});
		}
	}
};

export function useSSEEffect(chatId: () => number) {
	$inspect('messages', messages);
	$effect(() => {
		console.log('start sse');
		const id = chatId();
		const controller = new AbortController();

		RawAPIFetch<SseReq>('chat/sse', { id: id }, 'POST', controller.signal).then(
			async (response) => {
				if (response == undefined) return;

				const stream = events(response);

				for await (const event of stream) {
					const data = event.data;

					if (data != undefined && data.trim() != ':') {
						const resJson = JSON.parse(data) as SseResp;
						const error = getError(resJson);
						if (error) {
							dispatchError(error.error, error.reason);
						} else {
							const handler = Handlers[resJson.t];
							if (handler != undefined) handler(resJson.c as any, id);
						}
					} else {
						console.log(data);
					}
				}
			}
		);

		return () => {
			messages = [];
			version = -1;
			controller.abort();
		};
	});
}

async function syncMessages(chatId: number) {
	const resp = await APIFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', {
		t: 'limit',
		c: {
			chat_id: chatId,
			order: MessagePaginateReqOrder.Lt
		}
	});
	let streamingMessage = messages.filter((m) => m.stream);
	if (resp != undefined) messages = [...streamingMessage, ...resp.list];
}

function handleTokenChunk(
	kind: 'text' | 'reasoning' | 'tool_call' | 'tool_result' | 'error',
	chunkContent: any
) {
	const firstMsg = messages.at(0);
	if (!firstMsg || !firstMsg.stream) return;

	const lastChunk = firstMsg.chunks[firstMsg.chunks.length - 1];

	if (kind === 'text') {
		if (lastChunk && lastChunk.kind.t === 'text') {
			lastChunk.kind.c.content += chunkContent.content;
		} else {
			firstMsg.chunks.push({
				id: Date.now(),
				kind: { t: 'text', c: { content: chunkContent.content } }
			});
		}
	} else if (kind === 'reasoning') {
		if (lastChunk && lastChunk.kind.t === 'reasoning') {
			lastChunk.kind.c.content += chunkContent.content;
		} else {
			firstMsg.chunks.push({
				id: Date.now(),
				kind: { t: 'reasoning', c: { content: chunkContent.content } }
			});
		}
	} else if (kind === 'tool_call') {
		firstMsg.chunks.push({
			id: Date.now(),
			kind: { t: 'tool_call', c: chunkContent }
		});
	} else if (kind === 'tool_result') {
		if (lastChunk && lastChunk.kind.t === 'tool_call') {
			lastChunk.kind.c.content += chunkContent.content;
		} else {
			console.warn('Unexpected tool result without preceding tool call');
		}
	} else if (kind === 'error') {
		firstMsg.chunks.push({
			id: Date.now(),
			kind: { t: 'error', c: { content: chunkContent.content } }
		});
	}
}

export function getMessages() {
	return messages;
}

export function getStream() {
	return streaming;
}

export function pushUserMessage(
	id: number,
	user_id: number,
	content: string,
	files: MessagePaginateRespChunkKindFile[]
) {
	console.log('push user message');
	let fileChunks = files.map((f) => ({
		id: 0,
		kind: {
			t: 'file',
			c: f
		} as MessagePaginateRespChunkKind
	}));

	let streamingMessage = untrack(() => messages).filter((x) => x.stream);
	streamingMessage.splice(streamingMessage.length, 0, {
		id: user_id,
		chunks: [{ id: -1, kind: { t: 'text', c: { content } } }, ...fileChunks],
		role: MessagePaginateRespRole.User,
		token: 0,
		price: 0
	});
}

export function createMessage(): MutationResult<MessageCreateReq, MessageCreateResp> {
	return CreateMutation({
		path: 'message/create',
		onSuccess: (data, param) => {
			pushUserMessage(data.id, data.user_id, param.text, param.files || []);
		}
	});
}

export function updateMessage(): RawMutationResult<
	MessageCreateReq & { msgId: number },
	MessageCreateResp
> {
	const { mutate: create } = createMessage();
	return CreateRawMutation({
		mutator: (param) => {
			return new Promise(async (resolve, reject) => {
				await APIFetch<MessageDeleteReq, MessageDeleteReq>('message/delete', {
					id: param.msgId
				});

				await create(param, resolve);
			});
		}
	});
}
