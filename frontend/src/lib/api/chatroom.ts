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
	type ChatPaginateReqLimit,
	ChatPaginateReqOrder
} from './types';
import {
	CreateInfiniteQuery,
	CreateRawMutation,
	type InfiniteQueryResult,
	type Page,
	type RawMutationResult
} from './state';
import { apiFetch } from './state/errorHandle';

export interface CreateRoomRequest {
	message: string;
	modelId: number;
	files: File[];
	mode: Mode;
}

export function createRoom(): RawMutationResult<CreateRoomRequest, ChatCreateResp> {
	return CreateRawMutation({
		mutator: async (param) => {
			let chatRes = await apiFetch<ChatCreateResp, ChatCreateReq>('chat/create', {
				model_id: param.modelId
			});
			if (!chatRes) return;

			await goto('/chat/' + encodeURIComponent(chatRes.id));

			await apiFetch<MessageCreateResp, MessageCreateReq>('message/create', {
				chat_id: chatRes.id,
				text: param.message
			});
			return chatRes;
		},
		onSuccess: (data) => {
			// TODO: push front the rooms pagination
			// TODO: push front the chat pagination(first message)
		}
	});
}

const max_size = 16;
class ChatPage implements Page<ChatPaginateRespList> {
	ids: number[] = [];
	normal: boolean;
	constructor(id?: number, normal = true) {
		this.normal = normal;
		if (id) this.ids = [id];
	}
	async fetch(): Promise<ChatPaginateRespList[] | undefined> {
		let limit: ChatPaginateReqLimit = this.normal
			? {
					id: this.ids.length != 0 ? this.ids.at(0)! + 1 : undefined,
					limit: max_size,
					order: ChatPaginateReqOrder.Lt
				}
			: {
					id: this.ids.at(-1),
					limit: max_size,
					order: ChatPaginateReqOrder.Gt
				};

		const res = await apiFetch<ChatPaginateResp, ChatPaginateReq>('chat/paginate', {
			t: 'limit',
			c: limit
		});
		if (!res) return;

		const list = res.list;
		this.ids = list.map((x) => x.id);
		return list;
	}
	nextPage(): Page<ChatPaginateRespList> | undefined {
		if (this.ids.length >= max_size) return new ChatPage(this.ids.at(-1)! - 1);
	}
	insertFront(data: ChatPaginateRespList): Page<ChatPaginateRespList> | undefined {
		if (this.ids.length >= max_size) return new ChatPage(data.id, false);

		this.ids.unshift(data.id);
	}
}

export function useRoom(): InfiniteQueryResult<ChatPaginateRespList> {
	return CreateInfiniteQuery({
		key: ['room'],
		firstPage: new ChatPage()
	});
}
