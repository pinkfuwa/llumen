import { events } from 'fetch-event-stream';

import { APIFetch, getError, RawAPIFetch } from './state/errorHandle';

import {
	createMutation,
	createRawMutation,
	type MutationResult,
	type RawMutationResult
} from './state';
import type {
	MessageDeleteReq,
	MessageCreateReq,
	MessageCreateResp,
	MessagePaginateResp,
	MessagePaginateReq,
	MessagePaginateRespList,
	SseReq,
	SseResp,
	FileMetadata,
	Deep,
	AssistantChunk,
	SseCursor
} from './types';
import { MessagePaginateReqOrder } from './types';
import { dispatchError } from '$lib/error';
import { updateInfiniteQueryDataById } from './state';
import { getRoomPages, setRoomPages } from './chatroom.svelte';
import { untrack } from 'svelte';
import { dev } from '$app/environment';

type Message = MessagePaginateRespList & { stream?: boolean };
type AssistantMessage = Message & { inner: { t: 'assistant'; c: AssistantChunk[] } };

// version only reflect per-message changes, which is the minimal unit that pagination API cares
// when version mismatch, revaildate messages
// check version when backend send version event
// update version on push event(start/completed event)
let version = $state(-1);
// cursor reflect per-chunk and per-token changes, which handled by SSE API
// index for chunk, offset for token(counting char, not a proper tokenized counting).
//
// cursor are used to check small different between versions
//
// For example:
// User: What's llumen -> version 1
// Assistant: -> version 2 index 0 offset 0
// <thinking>I have to<thinking> -> version 2 index 0 offset 8
// **Quick Answer:** -> version 2 index 1 offset 17
let cursor = $state<SseCursor | null>(null);

// sorted in descending order by id
let messages = $state<Array<Message>>([]);

// Push a message with id to messages array
//
// If same id exist, replace it
function pushMessage(m: Message) {
	let idx = messages.findIndex((message) => message.id <= m.id);
	if (idx === -1) messages.push(m);
	else {
		const sameId = messages[idx].id === m.id;
		messages.splice(idx, Number(sameId), m);
	}
}

// State for tracking deep research plan being built during streaming
let deepState = $state<{
	currentStepIndex: number;
	fullJson: string;
} | null>(null);

const Handlers: {
	[key in SseResp['t']]: (data: Extract<SseResp, { t: key }>['c'], chatId: number) => void;
} = {
	version(data, chatId) {
		if (version !== data) {
			version = data;
			cursor = null;
			syncMessages(chatId);
		}
	},

	// user message arrived
	start(data) {
		const message: Message = {
			id: data.id,
			inner: {
				t: 'assistant',
				c: []
			},
			token_count: 0,
			price: 0,
			stream: true
		};
		pushMessage(message);
		version = data.version;
		cursor = { index: 0, offset: 0 };
		deepState = null;
	},

	token(token) {
		const firstMsg = messages[0] as AssistantMessage;

		const lastChunk = firstMsg.inner.c.at(-1);

		if (lastChunk?.t === 'text') {
			lastChunk.c += token as string;
			cursor!.offset += (token as string).length;
		} else {
			firstMsg.inner.c.push({ t: 'text', c: token as string });
			cursor!.index++;
			cursor!.offset = (token as string).length;
		}
	},

	reasoning(reasoning) {
		const firstMsg = messages[0] as AssistantMessage;

		const lastChunk = firstMsg.inner.c.at(-1);

		if (lastChunk?.t === 'reasoning') {
			lastChunk.c += reasoning as string;
			cursor!.offset += (reasoning as string).length;
		} else {
			firstMsg.inner.c.push({ t: 'reasoning', c: reasoning as string });
			cursor!.index++;
			cursor!.offset = (reasoning as string).length;
		}
	},

	tool_call(toolCall) {
		const firstMsg = messages[0] as AssistantMessage;

		const toolCallObj = toolCall as { name: string; args: string };
		firstMsg.inner.c.push({
			t: 'tool_call',
			c: {
				id: Date.now().toString(),
				name: toolCallObj.name,
				arg: toolCallObj.args
			}
		});
		cursor!.index++;
		cursor!.offset = 0;
	},

	tool_result(toolResult) {
		handleToolResult(toolResult.content);
		cursor!.index++;
		cursor!.offset = 0;
	},

	complete(data) {
		const firstMsg = messages[0] as AssistantMessage;
		firstMsg.stream = false;
		firstMsg.token_count = data.token_count;
		firstMsg.price = data.cost;
		version = data.version;
		cursor = null;

		if (messages.length > 1) messages[1].stream = false;
	},

	title(data, chatId) {
		const pages = getRoomPages();
		setRoomPages(updateInfiniteQueryDataById(pages, chatId, (chat) => ({ ...chat, title: data })));
		cursor!.index++;
		cursor!.offset = 0;
	},

	error(err) {
		const firstMsg = messages[0] as AssistantMessage;

		if (firstMsg && firstMsg.stream) {
			firstMsg.inner.c.push({
				t: 'error',
				c: err
			});
			cursor!.index++;
			cursor!.offset = (err as string).length;
		}
	},

	deep_plan(planChunk) {
		const firstMsg = messages[0] as AssistantMessage;

		const lastChunk = firstMsg.inner.c.at(-1);

		if (!lastChunk || lastChunk.t != 'deep_agent') {
			firstMsg.inner.c.push({
				t: 'deep_agent',
				c: {
					locale: '',
					has_enough_context: false,
					thought: '',
					title: '',
					steps: []
				}
			});
		}
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;

		if (deepState) {
			cursor!.offset += (planChunk as string).length;
		} else {
			cursor!.index++;
			cursor!.offset = (planChunk as string).length;
		}

		// Initialize deepState if not already initialized
		if (!deepState) {
			deepState = {
				currentStepIndex: -1,
				fullJson: ''
			};
		}
		deepState!.fullJson += planChunk;
		try {
			const lastChunk = firstMsg.inner.c.at(-1)!;
			const agentCall = JSON.parse(deepState!.fullJson) as Deep;
			JSON.parse(deepState!.fullJson);
			for (const step of agentCall.steps) step.progress = [];
			lastChunk.c = agentCall;
		} catch (e) {
			console.warn('invaild plan');
		}
	},

	deep_step_start(stepIndex) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		if (!deepState) throw new Error('deepState is not initialized');
		const lastChunk = firstMsg.inner.c.at(-1)!;
		deepState.currentStepIndex = stepIndex as number;
		cursor!.index++;
		cursor!.offset = 0;
	},

	deep_step_token(token) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const lastChunk = step.progress.at(-1);
		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += token as string;
			cursor!.offset += (token as string).length;
		} else {
			step.progress.push({ t: 'text', c: token as string });
			cursor!.index++;
			cursor!.offset = (token as string).length;
		}
	},

	deep_step_reasoning(reasoning) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const lastChunk = step.progress.at(-1);
		if (lastChunk && lastChunk.t === 'reasoning') {
			lastChunk.c += reasoning as string;
			cursor!.offset += (reasoning as string).length;
		} else {
			step.progress.push({ t: 'reasoning', c: reasoning as string });
			cursor!.index++;
			cursor!.offset = (reasoning as string).length;
		}
	},

	deep_step_tool_call(toolCall) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const toolCallObj = toolCall as { name: string; args: string };
		step.progress.push({
			t: 'tool_call',
			c: {
				id: Date.now().toString(),
				name: toolCallObj.name,
				arg: toolCallObj.args
			}
		});
		cursor!.index++;
		cursor!.offset = 0;
	},

	deep_step_tool_result(toolResult) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const result = (toolResult as { content: string }).content;
		// Find the last tool_call in progress that doesn't have a matching tool_result yet
		for (let i = step.progress.length - 1; i >= 0; i--) {
			const chunk = step.progress[i];
			if (chunk.t === 'tool_call') {
				// Check if next chunk is already a tool_result for this tool_call
				const nextChunk = step.progress[i + 1];
				if (!nextChunk || nextChunk.t !== 'tool_result' || nextChunk.c.id !== chunk.c.id) {
					// Add tool_result right after the tool_call
					step.progress.splice(i + 1, 0, {
						t: 'tool_result',
						c: {
							id: chunk.c.id,
							response: result
						}
					});
					break;
				}
			}
		}
		cursor!.index++;
		cursor!.offset = 0;
	},

	deep_report(report) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

		const lastChunk = firstMsg.inner.c.at(-1);

		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += report as string;
			cursor!.offset += (report as string).length;
		} else {
			firstMsg.inner.c.push({ t: 'text', c: report as string });
			cursor!.index++;
			cursor!.offset = (report as string).length;
		}
	},

	image(fileId) {
		const firstMsg = messages[0] as AssistantMessage;

		firstMsg.inner.c.push({
			t: 'image',
			c: fileId as number
		});
		cursor!.index++;
		cursor!.offset = 0;
	}
};

function startSSE(chatId: number, signal: AbortSignal) {
	let cursor_ = untrack(() => cursor);

	const req: SseReq = { id: chatId };
	if (cursor_ != null) {
		req.resume = { cursor: cursor_, version: untrack(() => version) };
	}

	RawAPIFetch<SseReq>('chat/sse', req, 'POST', signal).then(async (response) => {
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
			console.log('SSE aborted', e);
		}
	});
}

export function useSSEEffect(chatId: () => number) {
	if (dev) {
		$inspect('messages', messages);
		$inspect('version', version);
	}
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
			cursor = { index: -1, offset: 0 };
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
		// Merge streaming and fetched messages, sorted by ID in descending order (newest first)
		messages = [...streamingMessage, ...resp.list].sort((a, b) => b.id - a.id);
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

export function getMessages(): {
	readonly messages: Message[];
} {
	return {
		get messages() {
			return messages;
		}
	};
}

export function getStream(): {
	readonly stream: boolean;
} {
	return {
		get stream() {
			return messages.at(0)?.stream ? true : false;
		}
	};
}

export function pushUserMessage(user_id: number, content: string, files: FileMetadata[]) {
	pushMessage({
		id: user_id,
		inner: {
			t: 'user',
			c: {
				text: content,
				files: files
			}
		},
		token_count: 0,
		price: 0,
		stream: true
	});
}

export function createMessage(): MutationResult<MessageCreateReq, MessageCreateResp> {
	return createMutation({
		path: 'message/create',
		onSuccess: (data, param) => {
			pushUserMessage(data.user_id, param.text, param.files || []);
		}
	});
}

export function deleteMessage(): MutationResult<MessageDeleteReq, MessageDeleteReq> {
	return createMutation({
		path: 'message/delete',
		onSuccess: (data, param) => {
			messages = messages.filter((x) => x.id < param.id);
		}
	});
}

export function updateMessage(): RawMutationResult<
	MessageCreateReq & { msgId: number },
	MessageCreateResp
> {
	const { mutate: create } = createMessage();
	return createRawMutation({
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
