import type { Mode } from './model';
import { goto } from '$app/navigation';
import {
	type ChatCreateReq,
	type ChatCreateResp,
	type MessageCreateResp,
	type MessageCreateReq,
	type ChatPaginateRespList,
	type ChatPaginateReq,
	type ChatPaginateResp,
	ChatPaginateReqOrder,
	type MessagePaginateRespList,
	MessagePaginateRespRole,
	type ChatReadResp,
	type ChatReadReq,
	type ChatDeleteReq,
	type ChatDeleteResp
} from './types';
import {
	CreateInfiniteQuery,
	CreateMutation,
	CreateQuery,
	CreateRawMutation,
	GetEventQueryStatus,
	SetInfiniteQueryData,
	type Fetcher,
	type InfiniteQueryResult,
	type QueryResult,
	type RawMutationResult
} from './state';
import { APIFetch } from './state/errorHandle';
import { once } from './state/helper';
import { onDestroy } from 'svelte';
import type { MutationResult } from './state/mutate';

export interface CreateRoomRequest {
	message: string;
	modelId: number;
	files: File[];
	mode: Mode;
}

export function createRoom(): RawMutationResult<CreateRoomRequest, ChatCreateResp> {
	return CreateRawMutation({
		mutator: async (param) => {
			let chatRes = await APIFetch<ChatCreateResp, ChatCreateReq>('chat/create', {
				model_id: param.modelId
			});

			if (!chatRes) return;

			SetInfiniteQueryData<ChatPaginateRespList>({
				key: ['chatPaginate'],
				data: {
					id: chatRes.id,
					model_id: param.modelId,
					title: 'new chat'
				}
			});

			const status = GetEventQueryStatus(['messageEvent', chatRes.id.toString()]);

			await goto('/chat/' + encodeURIComponent(chatRes.id));

			// TODO: here is a resource leak(callback should be call on next route change)
			const callback = once(
				status,
				(x) => x,
				async () => {
					const res = await APIFetch<MessageCreateResp, MessageCreateReq>('message/create', {
						chat_id: chatRes.id,
						text: param.message
					});
					if (!res) return;
					SetInfiniteQueryData<MessagePaginateRespList>({
						key: ['messagePaginate', chatRes.id.toString()],
						data: {
							id: res.id,
							text: param.message,
							role: MessagePaginateRespRole.User
						}
					});
				}
			);

			return chatRes;
		}
	});
}

class ChatFetcher implements Fetcher<ChatPaginateRespList> {
	async range(startId: number, endId: number) {
		const x = await APIFetch<ChatPaginateResp, ChatPaginateReq>('chat/paginate', {
			t: 'range',
			c: {
				upper: startId + 1,
				lower: endId - 1
			}
		});
		return x?.list.sort((a, b) => b.id - a.id);
	}
	async forward(limit: number, id?: number) {
		if (id != undefined) id = id + 1;
		const x = await APIFetch<ChatPaginateResp, ChatPaginateReq>('chat/paginate', {
			t: 'limit',
			c: {
				id,
				limit,
				order: ChatPaginateReqOrder.Lt
			}
		});
		return x?.list.sort((a, b) => b.id - a.id);
	}
	async backward(limit: number, id: number) {
		if (id != undefined) id = id - 1;
		const x = await APIFetch<ChatPaginateResp, ChatPaginateReq>('chat/paginate', {
			t: 'limit',
			c: {
				id,
				limit,
				order: ChatPaginateReqOrder.Gt
			}
		});
		return x?.list.sort((a, b) => b.id - a.id);
	}
}

export function useRooms(): InfiniteQueryResult<ChatPaginateRespList> {
	return CreateInfiniteQuery({
		key: ['chatPaginate'],
		fetcher: new ChatFetcher()
	});
}

export function haltCompletion() {
	return CreateMutation({
		path: 'chat/halt',
		onSuccess: (data) => {
			// no need to update cache, SSE will handle it
		}
	});
}

export function useRoom(id: number): QueryResult<ChatReadResp> {
	return CreateQuery<ChatReadReq, ChatReadResp>({
		key: ['chatRead', id.toString()],
		path: 'chat/read',
		body: { id },
		revalidateOnFocus: false,
		staleTime: Infinity
	});
}

export function deleteRoom(): MutationResult<ChatDeleteReq, ChatDeleteResp> {
	return CreateMutation<ChatDeleteReq, ChatDeleteResp>({
		path: 'chat/delete'
	});
}
