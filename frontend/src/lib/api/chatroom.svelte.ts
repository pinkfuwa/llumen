import { page } from '$app/state';
import { untrack } from 'svelte';
import { goto } from '$app/navigation';
import { APIFetch } from './errorHandle.svelte';
import { dev } from '$app/environment';
import type { MutationStatus } from '.';
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
import { ChatPaginateReqOrder } from './types';
import { token } from '$lib/store.svelte';

export interface Entry {
	name: string;
	id: number;
}

function assertDescendingChatrooms(arr: Entry[]) {
	if (!dev) return;
	for (let i = 1; i < arr.length; i++) {
		if (arr[i - 1].id <= arr[i].id) {
			console.error('invariant: chatrooms not strictly descending at', i, arr[i - 1].id, arr[i].id);
		}
	}
}

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

// Module-level state for sidebar chatroom list. Array is sorted newest-first (descending id).
// Components bind to `chatrooms.val` directly and the scroll container sets `paginateElement`.
export const chatrooms = $state<{ val: Array<Entry> }>({ val: [] });

export const currentRoom = $state<{ val?: ChatReadResp }>({ val: undefined });

// Auto-fetch currentRoom when page.params.id changes.
$effect.root(() => {
	$effect(() => {
		void token.value?.value;

		const pid = page.params.id;
		if (!pid || isNaN(+pid)) {
			currentRoom.val = undefined;
			return;
		}
		const chatId = +pid;
		APIFetch<ChatReadResp, ChatReadReq>('chat/read', { id: chatId }).then((resp) => {
			if (resp) currentRoom.val = resp;
		});
	});
});

// rightPaginationAnchor: last id in array (smallest id = oldest).
// When chatrooms is empty and on a chat page, uses current chat id as pivot so
// the first fetch loads items around the active chat.
const rightPaginationAnchor = $derived.by(() => {
	if (chatrooms.val.length != 0) return chatrooms.val.at(-1)!.id;
	if (!page.route.id?.startsWith('/chat')) return Infinity;
	const pid = page.params.id;
	if (pid === undefined || isNaN(+pid)) return Infinity;
	return +pid;
});
// leftPaginationAnchor: first id in array (largest id = newest). undefined when empty.
const leftPaginationAnchor = $derived(chatrooms.val.at(0)?.id);

let leftExhausted = $state({ val: false });
let rightExhausted = $state({ val: false });

// Fetches items older than the rightmost entry (Lt = smaller id).
// When anchor is Infinity (empty list), omits id to get most recent items from backend.
async function fetchRight(): Promise<Array<Entry>> {
	const anchor = rightPaginationAnchor;
	if (rightExhausted.val || anchor === undefined) return [];
	const params: ChatPaginateReq =
		anchor === Infinity
			? { t: 'limit', c: { order: ChatPaginateReqOrder.Lt } }
			: { t: 'limit', c: { id: anchor, order: ChatPaginateReqOrder.Lt } };
	const resp = await APIFetch<ChatPaginateResp, ChatPaginateReq>('chat/paginate', params);
	if (!resp) return [];
	return resp.list.map((x) => ({ id: x.id, name: x.title ?? 'New Chat' }));
}

// Fetches items newer than the leftmost entry (Gt = larger id).
// Backend returns Gt in ascending order; reverses to maintain descending order.
async function fetchLeft(): Promise<Array<Entry>> {
	const anchor = leftPaginationAnchor;
	if (leftExhausted.val || anchor === undefined) return [];
	const resp = await APIFetch<ChatPaginateResp, ChatPaginateReq>('chat/paginate', {
		t: 'limit',
		c: { id: anchor, order: ChatPaginateReqOrder.Gt }
	});
	if (!resp) return [];
	return resp.list.map((x) => ({ id: x.id, name: x.title ?? 'New Chat' })).reverse();
}

// Reset exhaustion flags on visibility change so re-fetching happens after tab switch.
$effect.root(() => {
	document.addEventListener('visibilitychange', () => {
		leftExhausted.val = false;
		rightExhausted.val = false;
	});
});

let extending = false;

async function checkElement(target: HTMLElement, maxRecursion = 1) {
	if (maxRecursion === 0 || extending) return;
	let leftExtNeeded = target.scrollTop <= target.clientHeight * 0.8 && !leftExhausted.val;
	let rightExtNeeded =
		target.scrollHeight - target.scrollTop - target.clientHeight <= target.clientHeight * 0.8 &&
		!rightExhausted.val;

	extending = true;
	if (rightExtNeeded) {
		let ext = await fetchRight();

		extending = false;

		if (ext.length === 0) rightExhausted.val = true;
		else {
			chatrooms.val.push(...ext);
			assertDescendingChatrooms(chatrooms.val);
		}
		requestAnimationFrame(() => checkElement(target, maxRecursion - 1));
	} else if (leftExtNeeded) {
		let ext = await fetchLeft();
		extending = false;

		if (ext.length === 0) leftExhausted.val = true;
		else {
			const distanceFromBottom = target.scrollHeight - target.scrollTop;
			chatrooms.val.unshift(...ext);
			assertDescendingChatrooms(chatrooms.val);
			requestAnimationFrame(() => {
				target.scrollTop = target.scrollHeight - distanceFromBottom;
			});
		}
		requestAnimationFrame(() => checkElement(target, maxRecursion - 1));
	} else {
		extending = false;
	}
}

export const paginateElement = $state<{ val?: HTMLElement }>({ val: undefined });

function scrollEventHandler(e: Event) {
	untrack(() => checkElement(e.target as HTMLElement));
}

$effect.root(() => {
	$effect(() => {
		console.log('pag up', token.value?.value, paginateElement.val);
		void token.value?.value;
		const el = paginateElement.val;
		if (!el) return;
		leftExhausted.val = false;
		rightExhausted.val = false;
		untrack(() => checkElement(el, 50));
		el.addEventListener('scrollend', scrollEventHandler);
		return () => el.removeEventListener('scrollend', scrollEventHandler);
	});
});

// Uses Promise<MutationStatus> instead of createMutation pattern because
// the caller (Entry.svelte) reads the result synchronously to show UI state.
export function deleteEntry(id: number): Promise<MutationStatus> {
	return APIFetch<ChatDeleteResp, ChatDeleteReq>('chat/delete', { id }).then((resp) => {
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
	return APIFetch<ChatUpdateResp, ChatUpdateReq>('chat/write', { chat_id: id, title }).then(
		(resp) => {
			if (resp?.wrote) return 'success' as MutationStatus;
			return 'failed' as MutationStatus;
		}
	);
}

// Two-step flow: create chat room then immediately create the first message.
// Appends to sidebar before navigation so the room appears instantly.
export function createRoom(params: {
	message: string;
	modelId: number;
	files: Array<{ id: number; name: string }>;
	mode: ChatMode;
}): Promise<MutationStatus> {
	return APIFetch<ChatCreateResp, ChatCreateReq>('chat/create', {
		model_id: params.modelId,
		mode: params.mode
	}).then(async (resp) => {
		if (!resp) return 'failed';
		const chatId = resp.id;

		const msgResp = await APIFetch<MessageCreateResp, MessageCreateReq>('message/create', {
			chat_id: chatId,
			model_id: params.modelId,
			mode: params.mode,
			text: params.message,
			files: params.files as MessageCreateReqFile[]
		});
		if (!msgResp) return 'failed';

		chatrooms.val.unshift({ id: chatId, name: params.message });
		await goto(`/chat/${chatId}`);
		return 'success' as MutationStatus;
	});
}

// Called by message.svelte.ts SSE title handler. Fire-and-forget: updates sidebar
// optimistically; backend write is best-effort.
export function updateRoomTitle(chatId: number, title: string) {
	APIFetch<ChatUpdateResp, ChatUpdateReq>('chat/write', { chat_id: chatId, title });
	const entry = chatrooms.val.find((e) => e.id === chatId);
	if (entry) entry.name = title;

	if (currentRoom.val) currentRoom.val.title = title;
}

export function haltCompletion(params: { id: number }): Promise<unknown> {
	return APIFetch('chat/halt', params);
}
