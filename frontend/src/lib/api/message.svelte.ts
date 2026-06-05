import { events } from 'fetch-event-stream';
import { page } from '$app/state';

import { APIFetch, getError, RawAPIFetch } from './errorHandle.svelte';

import { FileKind, MessagePaginateReqOrder, ChatMode } from './types';
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
	ChatReadResp
} from './types';
import { displayError } from '$lib/error.svelte';
import { untrack } from 'svelte';
import { dev } from '$app/environment';
import { currentRoom, updateRoomTitle } from './chatroom.svelte';

// @typeshare generates MessagePaginateRespList without the stream flag (client-only).
type Message = MessagePaginateRespList & { stream?: boolean };
type AssistantMessage = Message & { inner: { t: 'assistant'; c: AssistantChunk[] } };

export const messages = $state<{ val: Array<Message> }>({ val: [] });
export const streaming = $state({ val: false });
export const paginateElement = $state<{ val?: HTMLDivElement }>({ val: undefined });
export const olderExhausted = $state({ val: false });

let version = $state(-1);
let streamingMessageId = $state<number | null>(null);

// index = chunk position within the assistant response
// offset = char offset within the text/reasoning chunk
let cursor = $state<SseCursor | null>(null);

function consumeDiscreteChunk() {
	cursor!.index++;
	cursor!.offset = 1;
}

function assertDescending(arr: Message[]) {
	if (!dev) return;
	for (let i = 1; i < arr.length; i++) {
		if (arr[i - 1].id <= arr[i].id) {
			console.error(
				'invariant: messages array not strictly descending at',
				i,
				arr[i - 1].id,
				arr[i].id
			);
		}
	}
}

function assertDescendingFetched(arr: Message[]) {
	if (!dev) return;
	for (let i = 1; i < arr.length; i++) {
		if (arr[i - 1].id <= arr[i].id) {
			console.error(
				'invariant: fetched array not strictly descending at',
				i,
				arr[i - 1].id,
				arr[i].id
			);
		}
	}
}

// First index where arr[idx].id <= target (descending array).
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

// First index where arr[idx].id < target (descending array).
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

// Merge two strictly descending arrays into one strictly descending array.
function mergeDescending(a: Message[], b: Message[]): Message[] {
	const result: Message[] = [];
	let i = 0,
		j = 0;
	while (i < a.length && j < b.length) {
		if (a[i].id >= b[j].id) {
			result.push(a[i]);
			i++;
		} else {
			result.push(b[j]);
			j++;
		}
	}
	result.push(...a.slice(i), ...b.slice(j));
	return result;
}

// Push/replace a message while preserving descending order (newest at index 0).
function pushMessage(m: Message) {
	const idx = firstLeIdx(messages.val, m.id);
	if (idx === messages.val.length) messages.val.push(m);
	else {
		const sameId = messages.val[idx].id === m.id;
		messages.val.splice(idx, Number(sameId), m);
	}
	assertDescending(messages.val);
}

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
		streamingMessageId = data.id;
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
			cursor!.offset += (token as string).length;
		} else {
			firstMsg.inner.c.push({ t: 'text', c: token as string });
			cursor!.index++;
			cursor!.offset = (token as string).length;
		}
	},

	reasoning(reasoning) {
		const firstMsg = messages.val[0] as AssistantMessage;
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
		streamingMessageId = null;
		version = data.version;
		cursor = null;
		if (messages.val.length > 1) messages.val[1].stream = false;
	},

	title(data, chatId) {
		updateRoomTitle(chatId, data);
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
			cursor!.offset = (err as string).length;
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
			cursor!.offset += (planChunk as string).length;
		} else {
			cursor!.index++;
			cursor!.offset = (planChunk as string).length;
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
			cursor!.offset += (token as string).length;
		} else {
			step.progress.push({ t: 'text', c: token as string });
			cursor!.index++;
			cursor!.offset = (token as string).length;
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
			cursor!.offset += (reasoning as string).length;
		} else {
			step.progress.push({ t: 'reasoning', c: reasoning as string });
			cursor!.index++;
			cursor!.offset = (reasoning as string).length;
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
			cursor!.offset += (report as string).length;
		} else {
			firstMsg.inner.c.push({ t: 'text', c: report as string });
			cursor!.index++;
			cursor!.offset = (report as string).length;
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

// At module-level: SSE auto-starts when page.params.id is set to a valid chat id.
// Cleanup runs on id change or module teardown.
$effect.root(() => {
	$effect(() => {
		const pid = page.params.id;
		if (!pid || isNaN(+pid)) return;
		const chatId = +pid;

		let controller = new AbortController();
		startSSE(chatId, controller.signal);

		function onVisibilityChange() {
			if (globalThis.document.visibilityState === 'visible') {
				if (!controller.signal.aborted) controller.abort();
				controller = new AbortController();
				startSSE(chatId, controller.signal);
			} else if (globalThis.document.visibilityState === 'hidden') {
				controller.abort();
			}
		}

		globalThis.document.addEventListener('visibilitychange', onVisibilityChange);

		return () => {
			globalThis.document.removeEventListener('visibilitychange', onVisibilityChange);
			messages.val = [];
			version = -1;
			streamingMessageId = null;
			cursor = { index: -1, offset: 0 };
			olderExhausted.val = false;
			controller.abort();
		};
	});

	// Bind scroll-based pagination to paginateElement.
	$effect(() => {
		void page.url.pathname;
		const el = paginateElement.val;
		if (!el) return;
		untrack(() => checkElement(el, 10));
		el.addEventListener('scrollend', scrollEventHandler);
		return () => el.removeEventListener('scrollend', scrollEventHandler);
	});
});

// Anchor: oldest message id in the array.
// When empty, uses Infinity so the first fetch returns the newest batch.
const olderPaginationAnchor = $derived.by(() => {
	if (messages.val.length !== 0) return messages.val.at(-1)!.id;
	return Infinity;
});

// Fetch messages older than the current oldest anchor.
// When anchor is Infinity (empty list), omits id to get most recent items.
async function fetchOlder(): Promise<Array<Message>> {
	const pid = page.params.id;
	if (!pid || isNaN(+pid)) return [];
	const chatId = +pid;
	if (olderExhausted.val) return [];

	const anchor = olderPaginationAnchor;
	if (anchor === undefined) return [];

	const params: MessagePaginateReq =
		anchor === Infinity
			? { t: 'limit', c: { chat_id: chatId, order: MessagePaginateReqOrder.Lt } }
			: { t: 'limit', c: { chat_id: chatId, id: anchor, order: MessagePaginateReqOrder.Lt } };
	const resp = await APIFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', params);
	if (!resp) return [];
	return resp.list;
}

// Re-sync on version mismatch. Merges fetched messages into existing array
// without dropping paginated history (messages with ids older than the fetch window).
async function syncMessages(chatId: number) {
	if (messages.val.length === 0) return;

	const resp = await APIFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', {
		t: 'limit',
		c: {
			chat_id: chatId,
			order: MessagePaginateReqOrder.Lt
		}
	});
	if (!resp) return;

	assertDescendingFetched(resp.list);

	const fetched = resp.list;

	// Merge-join two strictly descending arrays, deduplicating by id.
	const result: Message[] = [];
	let i = 0,
		j = 0;
	while (i < messages.val.length && j < fetched.length) {
		const msg = messages.val[i];
		const fet = fetched[j];
		if (msg.id > fet.id) {
			// msg not yet in fetched → keep our in-memory version
			result.push(msg);
			i++;
		} else if (msg.id < fet.id) {
			// fetched entry not in our array → take backend version
			result.push(fet);
			j++;
		} else {
			// same id: keep our in-memory version only if it's the streaming message
			if (msg.id === streamingMessageId) {
				result.push(msg);
				// skip the fetched duplicate
				const dupStart = j;
				while (j < fetched.length && fetched[j].id === msg.id) j++;
				if (j === dupStart) j++;
			} else {
				result.push(fet);
				j++;
			}
			i++;
		}
	}
	result.push(...messages.val.slice(i), ...fetched.slice(j));
	messages.val = result;
	assertDescending(messages.val);
}

// Reset exhaustion flags on visibility change so re-fetching happens after tab switch.
$effect.root(() => {
	document.addEventListener('visibilitychange', () => {
		olderExhausted.val = false;
	});
});

let extending = false;

async function checkElement(target: HTMLDivElement, maxRecursion = 1) {
	if (maxRecursion === 0 || extending) return;
	const olderNeeded = target.scrollTop <= target.clientHeight * 0.8 && !olderExhausted.val;

	if (!olderNeeded) return;

	extending = true;
	const prevScrollHeight = target.scrollHeight;
	let ext = await fetchOlder();
	extending = false;

	if (ext.length === 0) olderExhausted.val = true;
	else {
		messages.val.push(...ext);
		target.scrollTop = target.scrollHeight - prevScrollHeight;
	}
	checkElement(target, maxRecursion - 1);
}

function scrollEventHandler(e: Event) {
	untrack(() => checkElement(e.target as HTMLDivElement));
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

export async function createMessage(params: MessageCreateReq): Promise<void> {
	const resp = await APIFetch<MessageCreateResp, MessageCreateReq>('message/create', params);
	if (resp) pushUserMessage(resp.user_id, params.text, params.files || []);
}

export async function deleteMessage(id: number): Promise<void> {
	await APIFetch<MessageDeleteReq, MessageDeleteReq>('message/delete', { id });
	const firstKeepIdx = firstLtIdx(messages.val, id);
	if (firstKeepIdx === messages.val.length) messages.val.splice(0);
	else messages.val.splice(0, firstKeepIdx);
	assertDescending(messages.val);
}

export async function updateMessage(
	msgId: number,
	text: string,
	files: Array<{ id: number; name: string }>
): Promise<void> {
	const pid = page.params.id;
	if (!pid) return;
	const chatId = +pid;

	const room = currentRoom.val;
	if (!room || !room.model_id) {
		displayError('internal', 'select model first');
		return;
	}

	await APIFetch<MessageDeleteReq, MessageDeleteReq>('message/delete', { id: msgId });

	const firstKeepIdx = firstLtIdx(messages.val, msgId);
	if (firstKeepIdx === messages.val.length) messages.val.splice(0);
	else messages.val.splice(0, firstKeepIdx);

	const resp = await APIFetch<MessageCreateResp, MessageCreateReq>('message/create', {
		chat_id: chatId,
		model_id: room.model_id,
		mode: room.mode,
		text,
		files
	});
	if (resp) pushUserMessage(resp.user_id, text, files);
}

let pathname = $state(page.url.pathname);
$effect.root(() => {
	$effect(() => {
		if (page.url.pathname == pathname) return;
		untrack(() => {
			pathname = page.url.pathname;
		});
		console.log('cleaned!', page.url.pathname);
		streaming.val = false;
		messages.val = [];
		olderExhausted.val = false;
		streamingMessageId = null;
		version = -1;
	});
});

$effect.root(() => {
	let lastKey = $state(-1);
	let key = $derived(messages.val.at(-1)?.id || -2);
	$effect(() => {
		const el = paginateElement.val;
		if (!el) return;

		if (key !== lastKey) {
			untrack(() => (lastKey = key));
			requestAnimationFrame(() => {
				el.scrollTo({
					top: el.scrollHeight,
					behavior: 'instant'
				});
			});
		}
	});
});
