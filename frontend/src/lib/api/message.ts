import { CreateInfiniteQuery, CreateMutation, type InfiniteQueryResult, type Page } from './state';
import { apiFetch } from './state/errorHandle';
import type { MutationResult } from './state/mutate';
import {
	MessagePaginateReqOrder,
	type MessageCreateReq,
	type MessageCreateResp,
	type MessagePaginateReq,
	type MessagePaginateReqLimit,
	type MessagePaginateResp,
	type MessagePaginateRespList
} from './types';

const max_size = 16;
class MessagesPage implements Page<MessagePaginateRespList> {
	chatId: number;
	ids: number[] = [];
	normal: boolean;
	constructor(chatId: number, id?: number, normal = true) {
		this.chatId = chatId;
		this.normal = normal;
		if (id != undefined) this.ids = [id];
	}
	async fetch(): Promise<MessagePaginateRespList[] | undefined> {
		let limit: MessagePaginateReqLimit = this.normal
			? {
					chat_id: this.chatId,
					id: this.ids.length != 0 ? this.ids.at(0)! + 1 : undefined,
					limit: max_size,
					order: MessagePaginateReqOrder.Lt
				}
			: {
					chat_id: this.chatId,
					id: this.ids.at(-1),
					limit: max_size,
					order: MessagePaginateReqOrder.Gt
				};

		const res = await apiFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', {
			t: 'limit',
			c: limit
		});
		if (!res) return;

		const list = res.list;
		if (list.length != 0) this.ids = list.map((x) => x.id);
		return list;
	}
	nextPage(): Page<MessagePaginateRespList> | undefined {
		if (this.ids.length >= max_size) return new MessagesPage(this.chatId, this.ids.at(-1)! - 1);
	}
	insertFront(data: MessagePaginateRespList): Page<MessagePaginateRespList> | undefined {
		if (this.ids.length >= max_size) return new MessagesPage(this.chatId, data.id, false);

		this.ids.unshift(data.id);
	}
}

export function useMessage(chat_id: number): InfiniteQueryResult<MessagePaginateRespList> {
	return CreateInfiniteQuery({
		key: ['chat', chat_id.toString()],
		firstPage: new MessagesPage(chat_id)
	});
}

export function createMessage(): MutationResult<MessageCreateReq, MessageCreateResp> {
	return CreateMutation({
		path: 'message/create',
		onSuccess: () => {
			// TODO: push front the chat pagination
		}
	});
}
