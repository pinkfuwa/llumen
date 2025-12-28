import { goto } from '$app/navigation';
import type {
	ChatCreateReq,
	ChatCreateResp,
	MessageCreateResp,
	MessageCreateReq,
	ChatPaginateRespList,
	ChatPaginateReq,
	ChatPaginateResp,
	ChatReadResp,
	ChatReadReq,
	ChatDeleteReq,
	ChatDeleteResp,
	ChatUpdateReq,
	ChatUpdateResp
} from './types';
import { ChatPaginateReqOrder, ChatMode } from './types';
import {
	createInfiniteQueryEffect,
	createMutation,
	createRawMutation,
	createQueryEffect,
	insertInfiniteQueryData,
	updateInfiniteQueryDataById,
	type Fetcher,
	type MutationResult,
	type PageState,
	type RawMutationResult
} from './state';
import { APIFetch } from './state/errorHandle';
import { pushUserMessage } from './message.svelte';
import { dev } from '$app/environment';

export interface CreateRoomRequest {
	message: string;
	modelId: number;
	files: {
		name: string;
		id: number;
	}[];
	mode: ChatMode;
}

// Module-level state for infinite query
let roomPages = $state<PageState<ChatPaginateRespList>[]>([]);

// Module-level state for individual room
let currentRoom = $state<ChatReadResp | undefined>(undefined);
let currentRoomId = $state<number | undefined>(undefined);

// Query effects
export function useRoomsQueryEffect() {
	if (dev) $inspect('roomPages', roomPages);
	createInfiniteQueryEffect<ChatPaginateRespList>({
		fetcher: new ChatFetcher(),
		updatePages: (updater) => {
			roomPages = updater(roomPages);
		},
		getPages: () => roomPages,
		revalidateOnFocus: 'force'
	});
}

export function useRoomQueryEffect(id: number) {
	currentRoomId = id;
	createQueryEffect<ChatReadReq, ChatReadResp>({
		path: 'chat/read',
		body: { id },
		staleTime: Infinity,
		updateData: (data) => {
			if (currentRoomId === id) {
				currentRoom = data;
			}
		}
	});
}

// Getters
export function getRoomPages(): PageState<ChatPaginateRespList>[] {
	return roomPages;
}

export function getCurrentRoom(): ChatReadResp | undefined {
	return currentRoom;
}

// Setters
export function setRoomPages(pages: PageState<ChatPaginateRespList>[]) {
	roomPages = pages;
}

export function setCurrentRoom(data: ChatReadResp | undefined) {
	currentRoom = data;
}

// Mutations
export function createRoom(): RawMutationResult<CreateRoomRequest, ChatCreateResp> {
	return createRawMutation({
		mutator: async (param) => {
			let chatRes = await APIFetch<ChatCreateResp, ChatCreateReq>('chat/create', {
				model_id: param.modelId,
				mode: param.mode
			});

			if (!chatRes) return;

			let chatId = chatRes.id;

			const res = await APIFetch<MessageCreateResp, MessageCreateReq>('message/create', {
				chat_id: chatRes.id,
				text: param.message,
				mode: param.mode,
				model_id: param.modelId,
				files: param.files
			});

			if (!res) return;

			pushUserMessage(res.user_id, param.message, param.files);

			// Insert the new room into the infinite query
			roomPages = insertInfiniteQueryData(roomPages, {
				id: chatId,
				model_id: param.modelId
			});

			await goto('/chat/' + encodeURIComponent(chatId));

			return chatRes;
		}
	});
}

export function haltCompletion() {
	return createMutation({
		path: 'chat/halt',
		onSuccess: (data) => {
			// no need to update cache, SSE will handle it
		}
	});
}

export function deleteRoom(): MutationResult<ChatDeleteReq, ChatDeleteResp> {
	return createMutation<ChatDeleteReq, ChatDeleteResp>({
		path: 'chat/delete'
	});
}

export function updateRoom(): MutationResult<ChatUpdateReq, ChatUpdateResp> {
	return createMutation({
		path: 'chat/write'
	});
}

export function updateRoomTitle(id: number, title: string) {
	roomPages = updateInfiniteQueryDataById(roomPages, id, (data) => {
		return { ...data, title };
	});
}

// Fetcher implementation
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
		if (id !== undefined) id = id + 1;
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
		if (id !== undefined) id = id - 1;
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
