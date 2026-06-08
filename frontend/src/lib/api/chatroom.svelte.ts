import { page } from '$app/state';
import { untrack } from 'svelte';
import { goto } from '$app/navigation';
import { APIFetch } from './http.svelte';
import { ChatPaginateReqOrder } from './types';
import { token } from '$lib/rune.svelte';

import { type MutationStatus } from '.';
import type {
	ChatReadResp,
	ChatReadReq,
	ChatCreateReq,
	ChatCreateResp,
	ChatDeleteReq,
	ChatDeleteResp,
	ChatUpdateReq,
	ChatUpdateResp,
	ChatPaginateReq,
	ChatPaginateResp,
	ChatMode,
	MessageCreateReqFile,
	MessageCreateReq,
	MessageCreateResp
} from './types';

export interface Entry {
	name: string;
	id: number;
}

export const chatrooms = $state<{ val: Array<Entry> }>({ val: [] });
export const currentRoom = $state<{ val?: ChatReadResp }>({ val: undefined });
export const paginateElement = $state<{ val?: HTMLElement }>({ val: undefined });
let leftExhausted = false;
let rightExhausted = false;
let paginateRunning = false;

function findEntryIdx(arr: Entry[], id: number): number {
	let lo = 0,
		hi = arr.length;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		if (arr[mid].id <= id) hi = mid;
		else lo = mid + 1;
	}
	return lo;
}

export function getChatId() {
	const id = page.params.id;
	if (!id || isNaN(+id)) return;

	return +id;
}

async function ensurePaginated(target: HTMLElement, token_: string) {
	if (paginateRunning) return;
	paginateRunning = true;
	for (let i = 0; i < 50; i++) {
		const [leftNeeded, rightNeeded] = await new Promise<[boolean, boolean]>((resolve) => {
			// We have to wait for svelte update to reflect scrollTop/scrollHeight changes
			requestAnimationFrame(() => {
				resolve([
					target.scrollTop <= target.clientHeight * 0.8 && !leftExhausted,
					target.scrollHeight - target.scrollTop - target.clientHeight <=
						target.clientHeight * 0.8 && !rightExhausted
				]);
			});
		});
		if (!leftNeeded && !rightNeeded) break;
		if (rightNeeded) {
			let params: ChatPaginateReq;
			if (chatrooms.val.length === 0) {
				const pid = page.params.id;
				const pivotId =
					pid !== undefined && !isNaN(+pid) && page.route.id?.startsWith('/chat') ? +pid : 0;
				params =
					pivotId !== 0
						? { t: 'limit', c: { id: pivotId, order: ChatPaginateReqOrder.Lt } }
						: { t: 'limit', c: { order: ChatPaginateReqOrder.Lt } };
			} else {
				params = {
					t: 'limit',
					c: { id: chatrooms.val.at(-1)!.id, order: ChatPaginateReqOrder.Lt }
				};
			}
			const resp = await APIFetch<ChatPaginateResp, ChatPaginateReq>({
				path: 'chat/paginate',
				body: params,
				token: token_
			});
			if (!resp || resp.list.length === 0) {
				rightExhausted = true;
				break;
			}
			chatrooms.val.push(...resp.list.map((x) => ({ id: x.id, name: x.title ?? 'New Chat' })));
		} else if (leftNeeded) {
			const anchor = chatrooms.val.at(0)?.id;
			if (anchor === undefined) {
				leftExhausted = true;
				break;
			}
			const resp = await APIFetch<ChatPaginateResp, ChatPaginateReq>({
				path: 'chat/paginate',
				body: { t: 'limit', c: { id: anchor, order: ChatPaginateReqOrder.Gt } },
				token: token_
			});
			if (!resp || resp.list.length === 0) {
				leftExhausted = true;
				break;
			}
			const distanceFromBottom = target.scrollHeight - target.scrollTop;
			chatrooms.val.unshift(
				...resp.list.map((x) => ({ id: x.id, name: x.title ?? 'New Chat' })).reverse()
			);
			target.scrollTop = target.scrollHeight - distanceFromBottom;
		}
	}
	paginateRunning = false;
}

export function deleteEntry(id: number): Promise<MutationStatus> {
	return APIFetch<ChatDeleteResp, ChatDeleteReq>({
		path: 'chat/delete',
		body: { id },
		token: token.value?.value
	}).then((resp) => {
		if (resp) {
			const idx = findEntryIdx(chatrooms.val, id);
			if (idx < chatrooms.val.length && chatrooms.val[idx].id === id) {
				chatrooms.val.splice(idx, 1);
			}
			return 'success' as MutationStatus;
		}
		return 'failed' as MutationStatus;
	});
}

export function syncEntry(id: number, title: string): Promise<MutationStatus> {
	return APIFetch<ChatUpdateResp, ChatUpdateReq>({
		path: 'chat/write',
		body: { chat_id: id, title },
		token: token.value?.value
	}).then((resp) => {
		if (resp?.wrote) return 'success' as MutationStatus;
		return 'failed' as MutationStatus;
	});
}

export async function createRoom(params: {
	message: string;
	modelId: number;
	files: Array<{ id: number; name: string }>;
	mode: ChatMode;
}): Promise<MutationStatus> {
	const token_ = token.value?.value;
	if (!token_) return 'failed';
	const roomResp = await APIFetch<ChatCreateResp, ChatCreateReq>({
		path: 'chat/create',
		body: { model_id: params.modelId, mode: params.mode },
		token: token.value?.value
	});
	if (!roomResp) return 'failed';
	const chatId = roomResp.id;

	const msgResp = await APIFetch<MessageCreateResp, MessageCreateReq>({
		path: 'message/create',
		body: {
			chat_id: chatId,
			model_id: params.modelId,
			mode: params.mode,
			text: params.message,
			files: params.files as MessageCreateReqFile[]
		},
		token: token.value?.value
	});
	if (!msgResp) return 'failed';

	chatrooms.val.unshift({ id: chatId, name: params.message });
	await goto(`/chat/${chatId}`);
	return 'success' as MutationStatus;
}

export function haltCompletion(params: { id: number }): Promise<unknown> {
	return APIFetch({ path: 'chat/halt', body: params, token: token.value?.value! });
}

$effect.root(() => {
	$effect(() => {
		const chatId = getChatId();
		if (chatId === undefined) return;
		APIFetch<ChatReadResp, ChatReadReq>({
			path: 'chat/read',
			body: { id: chatId },
			token: true
		}).then((resp) => {
			if (resp) currentRoom.val = resp;
		});
	});
});

$effect.root(() => {
	document.addEventListener('visibilitychange', () => {
		void token.value?.value;

		leftExhausted = false;
		rightExhausted = false;
	});
});

$effect.root(() => {
	$effect(() => {
		const token_ = token.value?.value;
		const element = paginateElement.val;
		if (!element || !token_) return;

		function scrollEventHandler(e: Event) {
			const el = e.target as HTMLElement;
			untrack(() => ensurePaginated(el, token_!));
		}

		if (chatrooms.val.length == 0) untrack(() => ensurePaginated(element, token_));

		element.addEventListener('scroll', scrollEventHandler);
		return () => element.removeEventListener('scroll', scrollEventHandler);
	});
});

$effect.root(() => {
	$effect(() => {
		if (!token.value?.value) chatrooms.val = [];
	});
});
