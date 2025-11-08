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
	FileMetadata
} from './types';
import { MessagePaginateReqOrder } from './types';
import { dispatchError } from '$lib/error';
import { UpdateInfiniteQueryDataById } from './state';
import { untrack } from 'svelte';
import { dev } from '$app/environment';

let version = $state(-1);

let messages = $state<Array<MessagePaginateRespList & { stream?: boolean }>>([]);

// let deepState

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
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

		const lastChunk = firstMsg.inner.c.at(-1);

		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += token as string;
		} else {
			firstMsg.inner.c.push({ t: 'text', c: token as string });
		}
	},

	reasoning(reasoning) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

		const lastChunk = firstMsg.inner.c.at(-1);

		if (lastChunk && lastChunk.t === 'reasoning') {
			lastChunk.c += reasoning as string;
		} else {
			firstMsg.inner.c.push({ t: 'reasoning', c: reasoning as string });
		}
	},

	tool_call(toolCall) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

		const toolCallObj = toolCall as { name: string; args: string };
		firstMsg.inner.c.push({
			t: 'tool_call',
			c: {
				id: Date.now().toString(),
				name: toolCallObj.name,
				arg: toolCallObj.args
			}
		});
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

	// TODO: use deep_state to record delta, you can assume between start/complete, there is only one plan
	deep_plan(plan) {
		// record plan string to deep_state
		// concatenated plan yield a json for plan
		// use oboe.js to handle json stream(ie. display title/steps when the json is streaming)
	},

	deep_step_start(step) {
		// start of step
	},

	deep_step_token(step) {
		// token of step
		// if last token po progress is text, concat string,
		// otherwise record token to end of progress
	},

	deep_step_reasoning(step) {
		// similar to token, but for reasoning
	},

	deep_step_tool_call(toolCall) {
		// tool call start, followed by a tool call result(or error chunk if critical error is encounter).
		// Please note that our protocol allow is to stream unordered
		// For example: call(A)->call(B)->result(A)->result(B)
		// reorder=> call(A)->result(A)->call(B)->result(B)
	},
	deep_step_tool_result(data) {
		// tool call result
		// Find last step progress of tool_call with result empty
	},

	deep_report(report) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

		const lastChunk = firstMsg.inner.c.at(-1);

		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += report as string;
		} else {
			firstMsg.inner.c.push({ t: 'text', c: report as string });
		}
	}
};

function startSSE(chatId: number, signal: AbortSignal) {
	RawAPIFetch<SseReq>('chat/sse', { id: chatId }, 'POST', signal).then(async (response) => {
		if (response == undefined) return;

		const stream = events(response);

		try {
			for await (const event of stream) {
				// Check if this connection has been aborted before processing the event
				if (signal.aborted) break;

				const data = event.data;

				if (data != undefined && data.trim() != ':') {
					const resJson = JSON.parse(data) as SseResp;
					const error = getError(resJson);
					if (error) dispatchError(error.error, error.reason);
					else {
						if (dev) console.log('resJson', resJson);
						(Handlers[resJson.t] as (data: any, chatId: number) => void)(resJson.c, chatId);
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
