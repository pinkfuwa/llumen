import {
	CreateInfiniteQuery,
	CreateMutation,
	KeysetPage,
	type InfiniteQueryResult,
	type Page
} from './state';
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

export function useMessage(chat_id: number): InfiniteQueryResult<MessagePaginateRespList> {
	return CreateInfiniteQuery({
		key: ['chat', chat_id.toString()],
		firstPage: new KeysetPage<MessagePaginateRespList>((limit, id) =>
			apiFetch<MessagePaginateResp, MessagePaginateReq>('message/paginate', {
				t: 'limit',
				c: {
					chat_id,
					id,
					limit,
					order: MessagePaginateReqOrder.Lt
				}
			}).then((x) => x?.list)
		)
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
