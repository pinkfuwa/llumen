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
	type ChatDeleteResp,
	type ChatUpdateReq,
	type ChatUpdateResp,
	ChatMode,
	type MessageCreateReqFile
} from './types';
import {
	CreateInfiniteQuery,
	CreateMutation,
	CreateQuery,
	CreateRawMutation,
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
import { globalCache } from './state/cache';
import type { Writable } from 'svelte/store';
import { UpdateInfiniteQueryDataById } from './state';
import { upload } from './files';

export interface CreateRoomRequest {
	message: string;
	modelId: number;
	files: File[];
	mode: ChatMode;
}

export function createRoom(): RawMutationResult<CreateRoomRequest, ChatCreateResp> {
	return CreateRawMutation({
		mutator: async (param) => {
			let chatRes = await APIFetch<ChatCreateResp, ChatCreateReq>('chat/create', {
				model_id: param.modelId,
				mode: param.mode
			});

			if (!chatRes) return;

			let chatId = chatRes.id;

			SetInfiniteQueryData<ChatPaginateRespList>({
				key: ['chatPaginate'],
				data: {
					id: chatId,
					model_id: param.modelId
				}
			});

			const roomStreamingState = globalCache.getOr(['chat', 'stream', chatId.toString()], false);

			await goto('/chat/' + encodeURIComponent(chatId));

			roomStreamingState.set(true);

			let files: MessageCreateReqFile[] = [];

			for (const file of param.files) {
				try {
					let id = await upload(file, chatId);
					if (id == null) break;
					files.push({
						name: file.name,
						id
					});
				} catch (e) {
					console.warn(e);
				}
			}

			const res = await APIFetch<MessageCreateResp, MessageCreateReq>('message/create', {
				chat_id: chatRes.id,
				text: param.message,
				mode: param.mode,
				model_id: param.modelId,
				files
			});

			if (!res) return;
			SetInfiniteQueryData<MessagePaginateRespList>({
				key: ['messagePaginate', chatRes.id.toString()],
				data: {
					id: res.id,
					chunks: [{ id: res.id, kind: { t: 'text', c: { content: param.message } } }],
					role: MessagePaginateRespRole.User,
					token: 0,
					price: 0
				} as MessagePaginateRespList
			});

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
		fetcher: new ChatFetcher(),
		staleTime: 1000 * 60,
		revalidateOnFocus: 'force'
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

export function useRoomStreamingState(id: number): Writable<boolean> {
	return globalCache.getOr(['chat', 'stream', id.toString()], false);
}

export function updateRoom(): MutationResult<ChatUpdateReq, ChatUpdateResp> {
	return CreateMutation({
		path: 'chat/write'
	});
}

export function updateRoomTitle(id: number, title: string) {
	console.log({ id, title });
	UpdateInfiniteQueryDataById<ChatPaginateRespList>({
		key: ['chatPaginate'],
		updater: (data) => {
			if (data.id === id) data.title = title;
			return data;
		},
		id
	});
}
