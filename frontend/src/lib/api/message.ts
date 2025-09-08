import type { TokensList } from 'marked';
import {
	CreateEventQuery,
	CreateInfiniteQuery,
	CreateMutation,
	RemoveInfiniteQueryData,
	RevalidateInfiniteQueryData,
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
import { MarkdownPatcher, type UIUpdater } from '$lib/markdown/patcher';

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
		fetcher: new MessageFetcher(chat_id)
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
					text: param.text,
					role: MessagePaginateRespRole.User
				}
			});
		}
	});
}

export function handleServerSideMessage(chatId: number, streamingUI: UIUpdater) {
	let patcher = new MarkdownPatcher(streamingUI);
	const handlers: {
		[key in SseResp['t']]: (data: Extract<SseResp, { t: key }>['c']) => void;
	} = {
		last(data) {
			// data.version
			RevalidateInfiniteQueryData({
				key: ['messagePaginate', chatId.toString()],
				predicate: (entry) => data.id <= entry.id
			});
		},
		token(data) {
			streamingUI.tick();
			patcher.feed(data.text);
		},
		end(data) {
			SetInfiniteQueryData<MessagePaginateRespList>({
				key: ['messagePaginate', chatId.toString()],
				data: {
					id: data.id,
					text: patcher.content,
					role: MessagePaginateRespRole.Assistant
				}
			});
			patcher.reset();
		},
		user_message(data) {
			streamingUI.tick();

			// RemoveInfiniteQueryData<MessagePaginateRespList>({
			// 	key: ['messagePaginate', chatId.toString()],
			// 	predicate(entry) {
			// 		return data.id <= entry.id;
			// 	}
			// });

			// SetInfiniteQueryData<MessagePaginateRespList>({
			// 	key: ['messagePaginate', chatId.toString()],
			// 	data: {
			// 		id: data.id,
			// 		text: data.text,
			// 		role: MessagePaginateRespRole.User
			// 	}
			// });

			// TODO: fix bug
			// consider case when creating chatroom
			// 1. create chat
			// 2. setup Status handle for SSE
			// 3. setup SSE
			// 4. recieve user_message event
			// 5. append user message
			// 6. because status handle, it append user message again
			// 7. you got two message with same id!
			RevalidateInfiniteQueryData({
				key: ['messagePaginate', chatId.toString()],
				predicate: (entry) => data.id <= entry.id
			});
		}
	};

	CreateEventQuery<SseResp, SseReq>({
		path: 'chat/sse',
		key: ['messageEvent', chatId.toString()],
		body: {
			id: chatId
		},
		onEvent: (res: SseResp) => {
			handlers[res.t](res.c as any);
		}
	});
}
