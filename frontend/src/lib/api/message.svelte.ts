import { events } from 'fetch-event-stream';
import { page } from '$app/state';

import { APIFetch, getError, RawAPIFetch } from './http.svelte';

import { FileKind, MessagePaginateReqOrder } from './types';
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
	SseCursor,
	UrlCitation,
	MessageDeleteResp
} from './types';
import { displayError } from '$lib/error.svelte';
import { untrack } from 'svelte';
import { dev } from '$app/environment';
import { chatrooms, currentRoom, getChatId } from './chatroom.svelte';
import { token } from '$lib/rune.svelte';
import type { MutationStatus } from '.';

// @typeshare generates MessagePaginateRespList without the stream flag (client-only).
type Message = MessagePaginateRespList & { stream?: boolean };
type AssistantMessage = Message & { inner: { t: 'assistant'; c: AssistantChunk[] } };

export const messages = $state<{ val: Array<Message> }>({ val: [] });
export const streaming = $state({ val: false });
export const paginateElement = $state<{ val?: HTMLDivElement }>({ val: undefined });
let deepState = $state<{
	currentStepIndex: number;
	fullJson: string;
} | null>(null);
let version = $state(-1);
let cursor = $state<SseCursor | null>(null);
let lastKey = $state(-1);
let pathname = page.url.pathname;

let paginateRunning = false;
let exhausted = false;
let sseController = new AbortController();
let paginateChainAbort: AbortController = new AbortController();

function byteLen(s: string): number {
	return new TextEncoder().encode(s).length;
}

function consumeDiscreteChunk() {
	cursor!.index++;
	cursor!.offset = 1;
}

function firstLeIdx(arr: Message[], target: number): number {
	let lo = 0,
		hi = arr.length;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		if (arr[mid].id <= target) hi = mid;
		else lo = mid + 1;
	}
	return lo;
}

function firstLtIdx(arr: Message[], target: number): number {
	let lo = 0,
		hi = arr.length;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		if (arr[mid].id < target) hi = mid;
		else lo = mid + 1;
	}
	return lo;
}

function pushMessage(m: Message) {
	const idx = firstLeIdx(messages.val, m.id);
	if (idx === messages.val.length) messages.val.push(m);
	else {
		const sameId = messages.val[idx].id === m.id;
		messages.val.splice(idx, Number(sameId), m);
	}
}

export function pushUserMessage(
	user_id: number,
	content: string,
	files: Array<{ name: string; id: number; kind?: FileKind }>
) {
	pushMessage({
		id: user_id,
		inner: {
			t: 'user',
			c: {
				text: content,
				files
			}
		},
		token_count: 0,
		price: 0,
		stream: true
	});
}

const Handlers: {
	[key in SseResp['t']]: (data: Extract<SseResp, { t: key }>['c'], chatId: number) => void;
} = {
	version(data, chatId) {
		if (version == -1) version = data;
		else if (version !== data) {
			exhausted = false;
			paginateRunning = false;
			version = data;
			streaming.val = false;
			cursor = null;
			messages.val = [];
		}
	},

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
		streaming.val = true;
		version = data.version;
		cursor = { index: 0, offset: 1 };
		deepState = null;
	},

	token(token) {
		const firstMsg = messages.val[0] as AssistantMessage;
		const lastChunk = firstMsg.inner.c.at(-1);
		if (lastChunk?.t === 'text') {
			lastChunk.c += token as string;
			cursor!.offset += byteLen(token as string);
		} else {
			firstMsg.inner.c.push({ t: 'text', c: token as string });
			cursor!.index++;
			cursor!.offset = byteLen(token as string);
		}
	},

	reasoning(reasoning) {
		const firstMsg = messages.val[0] as AssistantMessage;
		const lastChunk = firstMsg.inner.c.at(-1);
		if (lastChunk?.t === 'reasoning') {
			lastChunk.c += reasoning as string;
			cursor!.offset += byteLen(reasoning as string);
		} else {
			firstMsg.inner.c.push({ t: 'reasoning', c: reasoning as string });
			cursor!.index++;
			cursor!.offset = byteLen(reasoning as string);
		}
	},

	tool_call(toolCall) {
		const firstMsg = messages.val[0] as AssistantMessage;
		const toolCallObj = toolCall as { name: string; args: string };
		firstMsg.inner.c.push({
			t: 'tool_call',
			c: {
				id: Date.now().toString(),
				name: toolCallObj.name,
				arg: toolCallObj.args
			}
		});
		consumeDiscreteChunk();
	},

	tool_result(toolResult) {
		const payload = toolResult as { content: string; files?: FileMetadata[] };
		handleToolResult(payload.content, payload.files || []);
		consumeDiscreteChunk();
	},

	complete(data) {
		const firstMsg = messages.val[0] as AssistantMessage;
		firstMsg.stream = false;
		firstMsg.token_count = data.token_count;
		firstMsg.price = data.cost;
		streaming.val = false;
		version = data.version;
		cursor = null;
		if (messages.val.length > 1) messages.val[1].stream = false;
	},

	title(data, chatId) {
		// FIXME: binary search
		chatrooms.val.find((e) => e.id === chatId)!.name = data;
		consumeDiscreteChunk();
	},

	error(err) {
		const firstMsg = messages.val[0] as AssistantMessage;
		if (firstMsg && firstMsg.stream) {
			firstMsg.inner.c.push({
				t: 'error',
				c: err
			});
			cursor!.index++;
			cursor!.offset = byteLen(err as string);
		}
	},

	deep_plan(planChunk) {
		const firstMsg = messages.val[0] as AssistantMessage;
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
			cursor!.offset += byteLen(planChunk as string);
		} else {
			cursor!.index++;
			cursor!.offset = byteLen(planChunk as string);
		}
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
		const firstMsg = messages.val.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		if (!deepState) throw new Error('deepState is not initialized');
		const lastChunk = firstMsg.inner.c.at(-1)!;
		deepState.currentStepIndex = stepIndex as number;
		consumeDiscreteChunk();
	},

	deep_step_token(token) {
		const firstMsg = messages.val.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const lastChunk = step.progress.at(-1);
		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += token as string;
			cursor!.offset += byteLen(token as string);
		} else {
			step.progress.push({ t: 'text', c: token as string });
			cursor!.index++;
			cursor!.offset = byteLen(token as string);
		}
	},

	deep_step_reasoning(reasoning) {
		const firstMsg = messages.val.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const lastChunk = step.progress.at(-1);
		if (lastChunk && lastChunk.t === 'reasoning') {
			lastChunk.c += reasoning as string;
			cursor!.offset += byteLen(reasoning as string);
		} else {
			step.progress.push({ t: 'reasoning', c: reasoning as string });
			cursor!.index++;
			cursor!.offset = byteLen(reasoning as string);
		}
	},

	deep_step_tool_call(toolCall) {
		const firstMsg = messages.val.at(0);
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
		consumeDiscreteChunk();
	},

	deep_step_tool_result(toolResult) {
		const firstMsg = messages.val.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const payload = toolResult as { content: string; files?: FileMetadata[] };
		const result = payload.content;
		const files = payload.files || [];
		for (let i = step.progress.length - 1; i >= 0; i--) {
			const chunk = step.progress[i];
			if (chunk.t === 'tool_call') {
				const nextChunk = step.progress[i + 1];
				if (!nextChunk || nextChunk.t !== 'tool_result' || nextChunk.c.id !== chunk.c.id) {
					step.progress.splice(i + 1, 0, {
						t: 'tool_result',
						c: {
							id: chunk.c.id,
							response: result,
							files
						}
					});
					break;
				}
			}
		}
		consumeDiscreteChunk();
	},

	deep_report(report) {
		const firstMsg = messages.val.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		const lastChunk = firstMsg.inner.c.at(-1);
		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += report as string;
			cursor!.offset += byteLen(report as string);
		} else {
			firstMsg.inner.c.push({ t: 'text', c: report as string });
			cursor!.index++;
			cursor!.offset = byteLen(report as string);
		}
	},

	image(fileId) {
		const firstMsg = messages.val[0] as AssistantMessage;
		firstMsg.inner.c.push({
			t: 'image',
			c: fileId as number
		});
		consumeDiscreteChunk();
	},

	url_citation(citations) {
		const firstMsg = messages.val[0] as AssistantMessage;
		firstMsg.inner.c.push({
			t: 'url_citation',
			c: citations as UrlCitation[]
		});
		consumeDiscreteChunk();
	}
};

function startSSE(chatId: number, token: string) {
	if (!sseController.signal.aborted) sseController.abort();
	sseController = new AbortController();

	const signal = sseController?.signal!;
	let cursor_ = untrack(() => cursor);

	const req: SseReq = { id: chatId };
	if (cursor_ != null) {
		req.resume = { cursor: cursor_, version: untrack(() => version) };
	}

	RawAPIFetch<SseReq>({ path: 'chat/sse', body: req, signal, token }).then(async (response) => {
		if (response == undefined) return;

		const stream = events(response);

		try {
			for await (const event of stream) {
				if (signal.aborted) break;

				const data = event.data;

				if (data != undefined && data.trim() != ':') {
					const resJson = JSON.parse(data) as SseResp;
					const error = getError(resJson);
					if (error) displayError(error.error, error.reason);
					else {
						(Handlers[resJson.t] as (data: any, chatId: number) => void)(resJson.c, chatId);
					}
				} else {
					console.log(data);
				}
			}
		} catch (e) {
			if (dev) console.log('SSE aborted', e);
		}
	});
}

async function paginateOne(
	target: HTMLDivElement,
	chatId: number,
	token: string
): Promise<boolean> {
	const anchor = messages.val.at(-1)?.id;

	const prevScrollHeight = target.scrollHeight;

	const params: MessagePaginateReq = {
		t: 'limit',
		c: { chat_id: chatId, id: anchor, order: MessagePaginateReqOrder.Lt }
	};
	const resp = await APIFetch<MessagePaginateResp, MessagePaginateReq>({
		path: 'message/paginate',
		body: params,
		signal: paginateChainAbort.signal,
		token
	});

	if (!resp || resp.list.length === 0) return true;

	messages.val.push(...resp.list);
	target.scrollTop = target.scrollHeight - prevScrollHeight;
	return true;
}

async function ensurePaginated(
	target: HTMLDivElement,
	chatId: number,
	token: string
): Promise<void> {
	const signal = paginateChainAbort.signal;
	if (signal.aborted || paginateRunning) return;
	paginateRunning = true;
	// Ideally, we should keep a state of max try, and increase it by 10 every ensurePaginated fired
	for (let i = 0; i < 10; i++) {
		if (exhausted) break;
		const needMore: boolean = await new Promise((resolve) => {
			// We have to wait for svelte update to reflect scrollTop/scrollHeight changes
			requestAnimationFrame(() => {
				resolve(target.scrollTop <= target.clientHeight * 0.8);
			});
		});
		if (!needMore) break;
		try {
			exhausted = await paginateOne(target, chatId, token);
		} catch (_) {
			break;
		}
	}
	paginateRunning = false;
}

function handleToolResult(result: string, files: FileMetadata[] = []) {
	const firstMsg = messages.val.at(0);
	if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;

	const chunks = firstMsg.inner.c;
	const lastChunk = chunks[chunks.length - 1];

	if (lastChunk && lastChunk.t === 'tool_call') {
		chunks.push({
			t: 'tool_result',
			c: {
				id: lastChunk.c.id,
				response: result,
				files
			}
		});
	} else {
		console.warn('Unexpected tool result without preceding tool call');
	}
}

export async function createMessage(params: MessageCreateReq): Promise<MutationStatus> {
	const resp = await APIFetch<MessageCreateResp, MessageCreateReq>({
		path: 'message/create',
		body: params,
		token: token.value?.value
	});
	if (resp) pushUserMessage(resp.user_id, params.text, params.files);
	return resp ? 'success' : 'failed';
}

export async function deleteMessage(id: number): Promise<MutationStatus> {
	const resp = await APIFetch<MessageDeleteResp, MessageDeleteReq>({
		path: 'message/delete',
		body: { id },
		token: token.value?.value
	});
	const firstKeepIdx = firstLtIdx(messages.val, id);
	if (firstKeepIdx === messages.val.length) messages.val.splice(0);
	else messages.val.splice(0, firstKeepIdx);

	if (resp) return resp.deleted ? 'success' : 'failed';
	return 'failed';
}

export async function syncMessage(
	msgId: number,
	text: string,
	files: Array<{ id: number; name: string }>
): Promise<MutationStatus> {
	const chatId = getChatId();
	if (chatId === undefined) return 'failed';

	const room = currentRoom.val;
	console.log(room);
	if (!room || !room.model_id) {
		displayError('internal', 'select model first');
		return 'failed';
	}

	await APIFetch<MessageDeleteReq, MessageDeleteReq>({
		path: 'message/delete',
		body: { id: msgId },
		token: true
	});

	const firstKeepIdx = firstLtIdx(messages.val, msgId);
	if (firstKeepIdx === messages.val.length) messages.val.splice(0);
	else messages.val.splice(0, firstKeepIdx);

	const resp = await APIFetch<MessageCreateResp, MessageCreateReq>({
		path: 'message/create',
		body: {
			chat_id: chatId,
			model_id: room.model_id,
			mode: room.mode,
			text,
			files
		},
		token: true
	});
	if (resp) pushUserMessage(resp.user_id, text, files);

	return resp ? 'success' : 'failed';
}

$effect.root(() => {
	$effect(() => {
		const chatId = getChatId();
		const token_ = token.value?.value;
		if (chatId == undefined || !token_) return;

		startSSE(chatId, token_);

		function onVisibilityChange() {
			const chatId = getChatId();
			if (chatId == undefined) return;
			if (globalThis.document.visibilityState === 'visible') {
				startSSE(chatId, token_!);
			} else if (globalThis.document.visibilityState === 'hidden') {
				sseController.abort('b');
			}
		}

		globalThis.document.addEventListener('visibilitychange', onVisibilityChange);

		return () => globalThis.document.removeEventListener('visibilitychange', onVisibilityChange);
	});

	$effect(() => {
		const el = paginateElement.val;
		const chatId = getChatId();
		const token_ = token.value?.value;
		if (!el || chatId == undefined || !token_) return;

		if (messages.val.length == 0) {
			untrack(() => ensurePaginated(el, chatId, token_!));
		}

		function scrollEventHandler() {
			untrack(() => ensurePaginated(el!, chatId!, token_!));
		}

		el.addEventListener('scroll', scrollEventHandler);
		return () => el.removeEventListener('scroll', scrollEventHandler);
	});
});

$effect.root(() => {
	$effect(() => {
		if (page.url.pathname == pathname) return;
		pathname = page.url.pathname;
		streaming.val = false;
		messages.val = [];
		exhausted = false;
		version = -1;
	});
});

$effect.root(() => {
	let key = $derived(messages.val.at(0)?.id || -2);
	$effect(() => {
		const el = paginateElement.val;
		if (el && key !== lastKey) {
			untrack(() => (lastKey = key));
			requestAnimationFrame(() => (el.scrollTop = el.scrollHeight));
		}
	});
});
