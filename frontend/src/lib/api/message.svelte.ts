import { events } from 'fetch-event-stream';
import oboe from 'oboe';

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
	FileMetadata,
	Deep,
	Step
} from './types';
import { MessagePaginateReqOrder } from './types';
import { dispatchError } from '$lib/error';
import { UpdateInfiniteQueryDataById } from './state';
import { untrack } from 'svelte';
import { dev } from '$app/environment';

let version = $state(-1);

let messages = $state<Array<MessagePaginateRespList & { stream?: boolean }>>([]);

// State for tracking deep research plan being built during streaming
let deepState = $state<{
	currentStepIndex: number;
	oboeInstance: oboe.Oboe;
	fullJson: string;
} | null>(null);

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
		const message = messages.find((m) => m.id == data.id);
		if (message == undefined) {
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
		} else {
			message.inner.c = [];
		}
		version = data.version;
		// Reset deepState for the new message
		deepState = null;
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

	deep_plan(planChunk) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
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
		// Initialize deepState if not already initialized
		if (!deepState) {
			deepState = {
				currentStepIndex: -1,
				oboeInstance: oboe()
					.node('locale', function (value) {
						plan.locale = value;
					})
					.node('has_enough_context', function (value) {
						plan.has_enough_context = value;
					})
					.node('thought', function (value) {
						plan.thought = value;
					})
					.node('title', function (value, path) {
						if (path.length === 1) plan.title = value;
					})
					.node('steps[*]', function (step, path) {
						plan.steps.push({
							...step,
							progress: []
						});
					}),
				fullJson: ''
			};
		}
		deepState!.fullJson += planChunk;
		deepState!.oboeInstance.emit('data', planChunk);
	},

	deep_step_start(stepIndex) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		if (!deepState) throw new Error('deepState is not initialized');
		const lastChunk = firstMsg.inner.c.at(-1)!;
		if (deepState.currentStepIndex == -1) {
			const agentCall = JSON.parse(deepState!.fullJson) as Deep;
			for (const step of agentCall.steps) step.progress = [];
			lastChunk.c = agentCall;
		}
		deepState.currentStepIndex = stepIndex as number;
	},

	deep_step_token(token) {
		const firstMsg = messages.at(0);
		if (!firstMsg || !firstMsg.stream || firstMsg.inner.t !== 'assistant') return;
		let plan = firstMsg.inner.c.at(-1)!.c as Deep;
		const step = plan.steps[deepState!.currentStepIndex];
		const lastChunk = step.progress.at(-1);
		if (lastChunk && lastChunk.t === 'text') {
			lastChunk.c += token as string;
		} else {
			step.progress.push({ t: 'text', c: token as string });
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
		} else {
			step.progress.push({ t: 'reasoning', c: reasoning as string });
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
			console.log('SSE aborted', e);
		}
	});
}

export function useSSEEffect(chatId: () => number) {
	if (dev) $inspect('messages', messages);
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
