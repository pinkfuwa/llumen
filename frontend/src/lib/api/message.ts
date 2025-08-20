import {
	CreateInfiniteQuery,
	CreateMutation,
	type Fetcher,
	type InfiniteQueryResult
} from './state';
import { APIFetch } from './state/errorHandle';
import type { MutationResult } from './state/mutate';
import {
	MessagePaginateReqOrder,
	type MessageCreateReq,
	type MessageCreateResp,
	type MessagePaginateReq,
	type MessagePaginateResp,
	type MessagePaginateRespList
} from './types';

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
		return x?.list;
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
		return x?.list;
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
		return x?.list;
	}
}

export function useMessage(chat_id: number): InfiniteQueryResult<MessagePaginateRespList> {
	return CreateInfiniteQuery({
		key: ['chatPaginate', chat_id.toString()],
		fetcher: new MessageFetcher(chat_id)
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
