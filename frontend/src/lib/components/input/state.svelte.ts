import { page } from '$app/state';
import { currentRoom, createRoom, haltCompletion } from '$lib/api/chatroom.svelte';
import { models } from '$lib/api/model.svelte';
import { createMessage, streaming } from '$lib/api/message.svelte';
import { createUploadPipeline } from '$lib/api/files.svelte';
import { getSupportedFileExtensions } from './fileTypes';
import { ChatMode } from '$lib/api/types';
import type { ModelList } from '$lib/api/types';
import { localState } from '$lib/store.svelte';

// ---------------------------------------------------------------------------
// Base = room value when room exists, localStorage default otherwise
// ---------------------------------------------------------------------------

const defaultModelId = localState<string | null>('DefaultModelId', {
	defaultValue: () => null
});

const baseModelId = $derived(currentRoom.val?.model_id?.toString() ?? defaultModelId.value);
const baseMode = $derived(currentRoom.val?.mode ?? ChatMode.Normal);

// ---------------------------------------------------------------------------
// Override = user-chosen via dropdown (undefined = no override)
// ---------------------------------------------------------------------------

export const overridingModelId = $state<{ val?: string }>({});
export const overridingMode = $state<{ val?: ChatMode }>({});

// ---------------------------------------------------------------------------
// Display = override wins, otherwise base
// ---------------------------------------------------------------------------

export const displayModelId = $state<{ val: string | null }>({ val: null });
export const displayMode = $state<{ val: ChatMode }>({ val: ChatMode.Normal });

// ---------------------------------------------------------------------------
// Model capability for the current display model
// ---------------------------------------------------------------------------

export const currentModelCap = $state<{ val?: ModelList }>({});
export const allowMode = $state<{
	val: {
		search_enabled: boolean;
		deep_research: boolean;
		media_gen: boolean;
		audio_input: boolean;
		image_input: boolean;
	};
}>({
	val: {
		search_enabled: false,
		deep_research: false,
		media_gen: false,
		audio_input: false,
		image_input: false
	}
});
export const supportedMimes = $state<{ val: string[] }>({ val: [] });

$effect.root(() => {
	$effect(() => {
		displayModelId.val = overridingModelId.val ?? baseModelId;
	});

	$effect(() => {
		displayMode.val = overridingMode.val ?? baseMode;
	});

	$effect(() => {
		const id = displayModelId.val;
		currentModelCap.val = models.val?.list.find((m) => m.id.toString() === id) ?? undefined;
	});

	$effect(() => {
		const cap = currentModelCap.val;
		const available = models.val != null;
		allowMode.val = {
			search_enabled: available && cap?.search_enabled === true,
			deep_research: available && cap?.deep_research === true,
			media_gen: available && cap?.media_gen === true,
			audio_input: available && cap?.audio_input === true,
			image_input: available && cap?.image_input === true
		};
	});

	$effect(() => {
		supportedMimes.val = getSupportedFileExtensions(currentModelCap.val ?? undefined);
	});
});

// ---------------------------------------------------------------------------
// Content & files (kept across chatrooms / /chat/new)
// ---------------------------------------------------------------------------

export const inputContent = $state<{ val: string }>({ val: '' });
export const inputFiles: { val: File[] } = $state({ val: [] });
export const submitting = $state({ val: false });

export let ensureUploaded: () => Promise<{ name: string; id: number }[]>;

$effect.root(() => {
	ensureUploaded = createUploadPipeline(() => inputFiles.val);
});

// ---------------------------------------------------------------------------
// Auto-reset mode when model doesn't support it
// ---------------------------------------------------------------------------

$effect.root(() => {
	$effect(() => {
		const cap = currentModelCap.val;
		if (!cap) return;

		const mode = displayMode.val;
		if (mode === ChatMode.Research && !cap.deep_research) {
			overridingMode.val = ChatMode.Normal;
		} else if (mode === ChatMode.Media && !cap.media_gen) {
			overridingMode.val = ChatMode.Normal;
		} else if (mode === ChatMode.Search && !cap.search_enabled) {
			overridingMode.val = ChatMode.Normal;
		}
	});
});

// ---------------------------------------------------------------------------
// User-initiated changes — called by ModelSelector / ModeSelector
// ---------------------------------------------------------------------------

export function onModelChange(newModelId: string) {
	overridingModelId.val = newModelId;
}

export function onModeChange(newMode: ChatMode) {
	overridingMode.val = newMode;
}

// ---------------------------------------------------------------------------
// Submit dispatch — checks route, calls createMessage or createRoom
// ---------------------------------------------------------------------------

export async function submit() {
	if (submitting.val) return;
	if (streaming.val) return;
	if (displayModelId.val == null) return;
	if (inputContent.val === '' && inputFiles.val.length === 0) return;

	submitting.val = true;
	const text = inputContent.val;
	const files = await ensureUploaded();
	const mode = displayMode.val;
	const modelIdNum = parseInt(displayModelId.val);

	let ok = false;
	try {
		const pid = page.params.id;
		if (pid && !isNaN(+pid)) {
			await createMessage({
				chat_id: +pid,
				text,
				mode,
				model_id: modelIdNum,
				files
			});
			ok = true;
		} else {
			const status = await createRoom({
				message: text,
				modelId: modelIdNum,
				files,
				mode
			});
			ok = status === 'success';
		}
	} finally {
		if (ok) {
			inputContent.val = '';
			inputFiles.val = [];
		}
		submitting.val = false;
	}
}

// ---------------------------------------------------------------------------
// Abort streaming — called by Stop button
// ---------------------------------------------------------------------------

export function abortStream() {
	const pid = page.params.id;
	if (!pid || isNaN(+pid)) return;
	haltCompletion({ id: +pid });
}
