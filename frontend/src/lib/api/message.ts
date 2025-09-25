import {
	CreateEventQuery,
	CreateInfiniteQuery,
	CreateMutation,
	CreateRawMutation,
	RemoveInfiniteQueryData,
	SetInfiniteQueryData,
	type Fetcher,
	type InfiniteQueryResult
} from './state';
import { APIFetch } from './state/errorHandle';
import type { MutationResult, RawMutationResult } from './state/mutate';
import {
	type MessageDeleteReq,
	MessagePaginateReqOrder,
	MessagePaginateRespRole,
	type MessageCreateReq,
	type MessageCreateResp,
	type MessagePaginateReq,
	type MessagePaginateResp,
	type MessagePaginateRespChunk,
	type MessagePaginateRespList,
	type SseReq,
	type SseResp
} from './types';
import { onDestroy } from 'svelte';
import { dev } from '$app/environment';
import { globalCache } from './state/cache';

class MessageFetcher implements Fetcher<MessagePaginateRespList> {
	chatId: number;
	constructor(chatId: number) {
		this.chatId = chatId;
	}
	async range(startId: number, endId: number) {
		const x = await APIFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', {
			t: 'range',
			c: {
				chat_id: this.chatId,
				upper: startId + 1,
				lower: endId - 1
			}
		});
		return x?.list.sort((a, b) => b.id - a.id);
	}
	async forward(limit: number, id?: number) {
		if (id != undefined) id = id + 1;
		const x = await APIFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', {
			t: 'limit',
			c: {
				chat_id: this.chatId,

				id,
				limit,
				order: MessagePaginateReqOrder.Lt
			}
		});
		return x?.list.sort((a, b) => b.id - a.id);
	}
	async backward(limit: number, id: number) {
		if (id != undefined) id = id - 1;
		const x = await APIFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', {
			t: 'limit',
			c: {
				chat_id: this.chatId,

				id,
				limit,
				order: MessagePaginateReqOrder.Gt
			}
		});
		return x?.list.sort((a, b) => b.id - a.id);
	}
}

export function useMessage(chat_id: number): InfiniteQueryResult<MessagePaginateRespList> {
	return CreateInfiniteQuery({
		key: ['messagePaginate', chat_id.toString()],
		fetcher: new MessageFetcher(chat_id),
		staleTime: Infinity,
		revalidateOnFocus: true
	});
}

export function createMessage(): MutationResult<MessageCreateReq, MessageCreateResp> {
	return CreateMutation({
		path: 'message/create',
		onSuccess: (data, param) => {
			let fileChunks = param.files?.map(
				(f) =>
					({
						id: 0,
						kind: { t: 'file', c: { name: f.name, id: f.id } }
					}) as MessagePaginateRespChunk
			);

			const roomStreamingState = globalCache.getOr(
				['chat', 'stream', param.chat_id.toString()],
				false
			);
			roomStreamingState.set(true);

			SetInfiniteQueryData<MessagePaginateRespList>({
				key: ['messagePaginate', param.chat_id.toString()],
				data: {
					id: data.id,
					role: MessagePaginateRespRole.User,
					chunks: [{ id: data.id, kind: { t: 'text', c: { content: param.text } } }, ...fileChunks],
					token: 0,
					price: 0
				}
			});
		}
	});
}

let SSEHandlers: {
	[key in SseResp['t']]: Array<(data: Extract<SseResp, { t: key }>['c']) => void>;
} = {
	token: [],
	reasoning: [],
	complete: [],
	tool_call: [],
	tool_result: [],
	title: [],
	error: [],
	version: []
} satisfies {
	[key in SseResp['t']]: Array<(data: Extract<SseResp, { t: key }>['c']) => void>;
};

let SSEQueued: {
	[key in SseResp['t']]: Array<Extract<SseResp, { t: key }>['c']>;
} = {
	token: [],
	reasoning: [],
	complete: [],
	tool_call: [],
	tool_result: [],
	title: [],
	error: [],
	version: []
} satisfies {
	[key in SseResp['t']]: Array<Extract<SseResp, { t: key }>['c']>;
};

let onSSEConnect: Array<() => void> = [];

export function startSSE(chatId: number) {
	CreateEventQuery<SseResp, SseReq>({
		path: 'chat/sse',
		key: ['messageEvent', chatId.toString()],
		body: {
			id: chatId
		},
		onEvent: (res: SseResp) => {
			if (dev) console.log('SSE Event:', res);

			const handlers = SSEHandlers[res.t];

			if (handlers.length === 0) SSEQueued[res.t].push(res.c as any);
			else handlers.forEach((handler) => handler(res.c as any));
		},
		onConnected: () => {
			if (dev) console.log('SSE Connected');
			onSSEConnect.forEach((handler) => handler());
		}
	});
}

export function addSSEHandler<T extends SseResp['t']>(
	event: T | 'connect',
	handler: ((data: Extract<SseResp, { t: T }>['c']) => void) | (() => void)
) {
	if (event === 'connect') {
		onSSEConnect.push(handler as () => void);
		return;
	}

	SSEHandlers[event].push(handler as any);

	SSEQueued[event].forEach((data) => handler(data as any));
	SSEQueued[event] = [];

	onDestroy(() => {
		const index = SSEHandlers[event].indexOf(handler as any);
		if (index !== -1) {
			SSEHandlers[event].splice(index, 1);
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

				RemoveInfiniteQueryData<MessagePaginateRespList>({
					predicate(entry) {
						return entry.id >= param.msgId;
					},
					key: ['messagePaginate', param.chat_id.toString()]
				});
				await create(param, resolve);
			});
		}
	});
}
