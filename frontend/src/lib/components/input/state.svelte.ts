import { page } from '$app/state';
import { currentRoom, createRoom, haltCompletion } from '$lib/api';
import { models } from '$lib/api/model.svelte';
import { createMessage, streaming } from '$lib/api/message.svelte';
import { createUploadPipeline } from '$lib/api/files.svelte';
import { getSupportedFileExtensions, separateFiles } from './fileTypes';
import { ChatMode } from '$lib/api/types';
import { localState } from '$lib/rune.svelte';

export const chatNewModelId = localState<string | null>('DefaultModelId', {
	defaultValue: () => null
});

const baseModelId = $derived(
	page.route.id === '/chat/new'
		? chatNewModelId.value
		: (currentRoom.val?.model_id?.toString() ?? null)
);

const baseMode = $derived(
	page.route.id === '/chat/new' ? ChatMode.Normal : (currentRoom.val?.mode ?? ChatMode.Normal)
);

const baseModelValid = $derived(models.val?.some((x) => x.id == Number(baseModelId)));

export const overridingModelId = $state<{ val?: string }>({});
export const overridingMode = $state<{ val?: ChatMode }>({});
export const inputContent = $state<{ val: string }>({ val: '' });
export const inputFiles: { val: File[] } = $state({ val: [] });
export const submitting = $state({ val: false });
export const pendingFile = $state<{ val: File[] }>({ val: [] });
export const allowedUnsupportedFiles = $state<{ val: File[] }>({ val: [] });
export const isEditing = $state({ val: true });

export class InputState {
	modelId = $derived.by(() => {
		const ov = overridingModelId.val;
		if (ov !== undefined) return ov;
		if (baseModelValid) return baseModelId;
		return null;
	});
	mode = $derived.by(() => overridingMode.val ?? baseMode);
	currentModel = $derived(models.val?.find((m) => m.id.toString() === this.modelId) ?? null);
	allowMode = $derived.by(() => {
		const cap = this.currentModel;
		return {
			search_enabled: cap?.search_enabled === true,
			deep_research: cap?.deep_research === true,
			media_gen: cap?.media_gen === true
		};
	});
	modality = $derived.by(() => {
		const cap = this.currentModel;
		return {
			image_input: cap?.image_input === true,
			audio_input: cap?.audio_input === true,
			video_input: cap?.video_input === true,
			native_file_input: cap?.native_file_input === true,
			ocr_file_input: cap?.ocr_file_input === true
		};
	});
	supportedMimes = $derived(getSupportedFileExtensions(this.currentModel ?? undefined));
}

export const effective = new InputState();

export const unsupportedFilesModalOpen = $state({ val: false });

export let ensureUploaded: () => Promise<{ name: string; id: number }[]>;

$effect.root(() => {
	ensureUploaded = createUploadPipeline(() => inputFiles.val);

	$effect(() => {
		const cap = effective.currentModel;
		if (!cap) return;

		const mode = effective.mode;
		if (mode === ChatMode.Research && !cap.deep_research) {
			overridingMode.val = ChatMode.Normal;
		} else if (mode === ChatMode.Media && !cap.media_gen) {
			overridingMode.val = ChatMode.Normal;
		} else if (mode === ChatMode.Search && !cap.search_enabled) {
			overridingMode.val = ChatMode.Normal;
		}
	});
});

export function addFiles(newFiles: File[]) {
	const mimes = effective.supportedMimes;
	if (!mimes.length) {
		for (const f of newFiles) inputFiles.val.push(f);
		return;
	}

	const { supported, unsupported } = separateFiles(newFiles, mimes);
	const newUnsupported = unsupported.filter(
		(u: File) =>
			!allowedUnsupportedFiles.val.some(
				(a: File) => a.name === u.name && a.size === u.size && a.lastModified === u.lastModified
			)
	);

	if (newUnsupported.length > 0) {
		for (const f of supported) inputFiles.val.push(f);
		pendingFile.val = newUnsupported;
		unsupportedFilesModalOpen.val = true;
	} else {
		for (const f of supported) inputFiles.val.push(f);
	}
}

export function onModelChange(newModelId: string) {
	overridingModelId.val = newModelId;

	if (page.route.id == '/chat/new') {
		chatNewModelId.value = newModelId;
	}
}

export function onModeChange(newMode: ChatMode) {
	overridingMode.val = newMode;
}

export async function submit() {
	if (submitting.val) return;
	if (streaming.val) return;
	if (effective.modelId == null) return;
	if (inputContent.val === '' && inputFiles.val.length === 0) return;

	submitting.val = true;
	const text = inputContent.val;
	const files = await ensureUploaded();
	const mode = effective.mode;
	const modelIdNum = parseInt(effective.modelId ?? '');

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
			pendingFile.val = [];
			allowedUnsupportedFiles.val = [];
		}
		submitting.val = false;
	}
}

export function abortStream() {
	const pid = page.params.id;
	if (!pid || isNaN(+pid)) return;
	haltCompletion({ id: +pid });
}
