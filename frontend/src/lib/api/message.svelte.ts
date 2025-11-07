import { events } from 'fetch-event-stream';

import { APIFetch, getError, RawAPIFetch } from './state/errorHandle';

import { CreateMutation, CreateRawMutation } from './state';
import type { MutationResult, RawMutationResult } from './state/mutate';
import type {
	MessageDeleteReq,
	MessageCreateReq,
	MessageCreateResp,
	MessagePaginateResp,
	MessagePaginateReq,
	MessagePaginateRespList,
	SseReq,
	SseResp,
	AssistantChunk,
	FileMetadata
} from './types';
import { MessagePaginateReqOrder } from './types';
import { dispatchError } from '$lib/error';
import { UpdateInfiniteQueryDataById } from './state';
import { untrack } from 'svelte';

let version = $state(-1);

let messages = $state<Array<MessagePaginateRespList & { stream?: boolean }>>([]);

// let streaming = $derived(!!messages.at(0)?.stream);

const Handlers: {
	[key in SseResp['t']]: (data: Extract<SseResp, { t: key }>['c'], chatId: number) => void;
} = {
	version(data, chatId) {
		if (version !== data) {
			version = data;
			syncMessages(chatId);
		}
	},

	start(data) {
		if (!messages.some((m) => m.id == data.id)) {
			messages.unshift({
				id: data.id,
				inner: {
					t: 'assistant',
					c: []
				},
				token_count: 0,
				price: 0,
				stream: true
			});
		}
		version = data.version;
	},

	token(token) {
		handleTokenChunk('text', token);
	},

	reasoning(reasoning) {
		handleTokenChunk('reasoning', reasoning);
	},

	tool_call(toolCall) {
		handleTokenChunk('tool_call', toolCall);
	},

	tool_result(toolResult) {
		handleToolResult(toolResult.content);
	},

	complete(data) {
		const lastMsg = messages.at(0);
		if (lastMsg && lastMsg.stream) {
			lastMsg.stream = false;
			lastMsg.token_count = data.token_count;
			lastMsg.price = data.cost;
		}
		version = data.version;
	},

	title(data, chatId) {
		UpdateInfiniteQueryDataById({
			key: ['chatPaginate'],
			id: chatId,
			updater: (chat) => ({ ...chat, title: data })
		});
	},

	error(err) {
		const lastMsg = messages.at(0);
		if (lastMsg && lastMsg.stream) {
			if (lastMsg.inner.t == 'user') return;

			lastMsg.inner.c.push({
				t: 'error',
				c: err
			});
		}
	},

	deep_plan(plan) {
		handleTokenChunk('annotation', plan);
	},

	deep_step_start(step) {
		handleTokenChunk('annotation', step);
	},

	deep_step_token(step) {
		handleTokenChunk('annotation', step);
	},

	deep_step_reasoning(step) {
		handleTokenChunk('reasoning', step);
	},

	deep_step_tool_call(toolCall) {
		handleTokenChunk('tool_call', toolCall);
	},

	deep_step_tool_token(token) {
		// Handle tool token if needed
	},

	deep_report(report) {
		handleTokenChunk('annotation', report);
	},

	tool_token(token) {}
};

function startSSE(chatId: number, signal: AbortSignal) {
	RawAPIFetch<SseReq>('chat/sse', { id: chatId }, 'POST', signal).then(async (response) => {
		if (response == undefined) return;

		const stream = events(response);

		try {
			for await (const event of stream) {
				// Check if this connection has been aborted before processing the event
				// This prevents duplicate chunks when a new connection starts while old one is still processing
				if (signal.aborted) {
					console.log('SSE connection aborted, stopping event processing');
					break;
				}

				const data = event.data;

				if (data != undefined && data.trim() != ':') {
					const resJson = JSON.parse(data) as SseResp;
					const error = getError(resJson);
					if (error) {
						dispatchError(error.error, error.reason);
					} else {
						const handler = Handlers[resJson.t] as (data: any, chatId: number) => void;
						if (handler != undefined) handler(resJson.c, chatId);
					}
				} else {
					console.log(data);
				}
			}
		} catch (e) {
			console.log('SSE aborted');
		}
	});
}

export function useSSEEffect(chatId: () => number) {
	$inspect('messages', messages);
	$effect(() => {
		let controller = new AbortController();

		const id = chatId();

		startSSE(id, controller.signal);

		function onVisibilityChange() {
			const state = globalThis.document.visibilityState;
			if (state === 'visible') {
				if (!controller.signal.aborted) controller.abort();
				controller = new AbortController();
				startSSE(id, controller.signal);
			} else if (state === 'hidden') controller.abort();
		}

		globalThis.document.addEventListener('visibilitychange', onVisibilityChange);

		return () => {
			globalThis.document.removeEventListener('visibilitychange', onVisibilityChange);
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
	if (resp != undefined) {
		let streamingMessage = messages.filter((m) => m.stream && !resp.list.some((x) => x.id == m.id));
		messages = [...streamingMessage, ...resp.list];
	}
}

function handleTokenChunk(
	kind: AssistantChunk['t'],
	content: string | { name: string; args: string }
) {
	const firstMsg = messages.at(0);
	if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

	const chunks = firstMsg.inner.c;
	const lastChunk = chunks[chunks.length - 1];

	if (kind === 'text') {
		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += content as string;
		} else {
			chunks.push({ t: 'text', c: content as string });
		}
	} else if (kind === 'reasoning') {
		if (lastChunk && lastChunk.t === 'reasoning') {
			lastChunk.c += content as string;
		} else {
			chunks.push({ t: 'reasoning', c: content as string });
		}
	} else if (kind === 'annotation') {
		if (lastChunk && lastChunk.t === 'annotation') {
			lastChunk.c += content as string;
		} else {
			chunks.push({ t: 'annotation', c: content as string });
		}
	} else if (kind === 'tool_call') {
		const toolCall = content as { name: string; args: string };
		chunks.push({
			t: 'tool_call',
			c: {
				id: Date.now().toString(),
				name: toolCall.name,
				arg: toolCall.args
			}
		});
	} else if (kind === 'error') {
		chunks.push({ t: 'error', c: content as string });
	}
}

function handleToolResult(result: string) {
	const firstMsg = messages.at(0);
	if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

	const chunks = firstMsg.inner.c;
	const lastChunk = chunks[chunks.length - 1];

	if (lastChunk && lastChunk.t === 'tool_call') {
		chunks.push({
			t: 'tool_result',
			c: {
				id: lastChunk.c.id,
				response: result
			}
		});
	} else {
		console.warn('Unexpected tool result without preceding tool call');
	}
}

export function getMessages() {
	return messages;
}

export function getStream(updater: (x: boolean) => void) {
	$effect(() => {
		const stream = messages.at(0)?.stream ? true : false;
		updater(stream);
	});
}

export function pushUserMessage(user_id: number, content: string, files: FileMetadata[]) {
	let streamingMessage = untrack(() => messages).filter((x) => x.stream);

	messages.splice(streamingMessage.length, 0, {
		id: user_id,
		inner: {
			t: 'user',
			c: {
				text: content,
				files: files
			}
		},
		token_count: 0,
		price: 0
	});
}

export function createMessage(): MutationResult<MessageCreateReq, MessageCreateResp> {
	return CreateMutation({
		path: 'message/create',
		onSuccess: (data, param) => {
			pushUserMessage(data.user_id, param.text, param.files || []);
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

				messages = messages.filter((x) => x.id < param.msgId);

				await create(param, resolve);
			});
		}
	});
}
