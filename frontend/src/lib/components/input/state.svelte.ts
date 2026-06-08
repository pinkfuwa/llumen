import { page } from '$app/state';
import { currentRoom, createRoom, haltCompletion } from '$lib/api';
import { models } from '$lib/api/model.svelte';
import { createMessage, streaming } from '$lib/api/message.svelte';
import { createUploadPipeline } from '$lib/api/files.svelte';
import { getSupportedFileExtensions } from './fileTypes';
import { ChatMode } from '$lib/api/types';
import type { ModelList } from '$lib/api/types';
import { localState } from '$lib/rune.svelte';

const defaultModelId = localState<string | null>('DefaultModelId', {
	defaultValue: () => null
});

const baseModelId = $derived(currentRoom.val?.model_id?.toString() ?? defaultModelId.value);
const baseMode = $derived(currentRoom.val?.mode ?? ChatMode.Normal);

export const overridingModelId = $state<{ val?: string }>({});
export const overridingMode = $state<{ val?: ChatMode }>({});

export const displayModelId = $state<{ val: string | null }>({ val: null });
export const displayMode = $state<{ val: ChatMode }>({ val: ChatMode.Normal });

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

const baseModelValid = $derived(models.val?.some((x) => x.id == Number(baseModelId)));

$effect.root(() => {
	$effect(() => {
		if (overridingModelId.val !== undefined) displayModelId.val = overridingModelId.val;
		else if (baseModelValid) {
			displayModelId.val = baseModelId;
		} else displayModelId.val = null;
	});

	$effect(() => {
		displayMode.val = overridingMode.val ?? baseMode;
	});

	$effect(() => {
		const id = displayModelId.val;
		currentModelCap.val = models.val?.find((m) => m.id.toString() === id) ?? undefined;
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

export const inputContent = $state<{ val: string }>({ val: '' });
export const inputFiles: { val: File[] } = $state({ val: [] });
export const submitting = $state({ val: false });

export let ensureUploaded: () => Promise<{ name: string; id: number }[]>;

$effect.root(() => {
	ensureUploaded = createUploadPipeline(() => inputFiles.val);
});

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

export function onModelChange(newModelId: string) {
	overridingModelId.val = newModelId;

	if (page.route.id == '/chat/new') {
		defaultModelId.value = newModelId;
	}
}

export function onModeChange(newMode: ChatMode) {
	overridingMode.val = newMode;
}

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

export function abortStream() {
	const pid = page.params.id;
	if (!pid || isNaN(+pid)) return;
	haltCompletion({ id: +pid });
}
