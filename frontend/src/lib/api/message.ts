import {
	CreateEventQuery,
	CreateInfiniteQuery,
	CreateMutation,
	SetInfiniteQueryData,
	type Fetcher,
	type InfiniteQueryResult
} from './state';
import { APIFetch } from './state/errorHandle';
import type { MutationResult } from './state/mutate';
import {
	MessagePaginateReqOrder,
	MessagePaginateRespRole,
	type MessageCreateReq,
	type MessageCreateResp,
	type MessagePaginateReq,
	type MessagePaginateResp,
	type MessagePaginateRespList,
	type SseReq,
	type SseResp
} from './types';
import { onDestroy } from 'svelte';
import { dev } from '$app/environment';

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
			SetInfiniteQueryData<MessagePaginateRespList>({
				key: ['messagePaginate', param.chat_id.toString()],
				data: {
					id: data.id,
					role: MessagePaginateRespRole.User,
					// TODO: fix chunk ID
					chunks: [{ id: data.id, kind: { t: 'text', c: { content: param.text } } }],
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

export function startSSE(chatId: number) {
	CreateEventQuery<SseResp, SseReq>({
		path: 'chat/sse',
		key: ['messageEvent', chatId.toString()],
		body: {
			id: chatId
		},
		onEvent: (res: SseResp) => {
			if (dev) console.log('SSE Event:', res);

			SSEHandlers[res.t].forEach((handler) => handler(res.c as any));
		}
	});
}

export function addSSEHandler<T extends SseResp['t']>(
	event: T,
	handler: (data: Extract<SseResp, { t: T }>['c']) => void
) {
	SSEHandlers[event].push(handler as any);

	onDestroy(() => {
		const index = SSEHandlers[event].indexOf(handler as any);
		if (index !== -1) {
			SSEHandlers[event].splice(index, 1);
		}
	});
}
