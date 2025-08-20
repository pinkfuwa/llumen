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
	ChatPaginateReqOrder
} from './types';
import {
	CreateInfiniteQuery,
	CreateRawMutation,
	SetInfiniteQueryData,
	type Fetcher,
	type InfiniteQueryResult,
	type RawMutationResult
} from './state';
import { APIFetch } from './state/errorHandle';

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

			await goto('/chat/' + encodeURIComponent(chatRes.id));

			await APIFetch<MessageCreateResp, MessageCreateReq>('message/create', {
				chat_id: chatRes.id,
				text: param.message
			});
			return chatRes;
		},
		onSuccess: (data, param) => {
			SetInfiniteQueryData<ChatPaginateRespList>({
				key: ['chatPaginate'],
				data: {
					id: data.id,
					model_id: param.modelId,
					title: 'new chat'
				}
			});
			// TODO: push front the rooms pagination
			// TODO: push front the chat pagination(first message)
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
		return x?.list;
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
		return x?.list;
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
		return x?.list;
	}
}

export function useRoom(): InfiniteQueryResult<ChatPaginateRespList> {
	return CreateInfiniteQuery({
		key: ['chatPaginate'],
		fetcher: new ChatFetcher()
	});
}
