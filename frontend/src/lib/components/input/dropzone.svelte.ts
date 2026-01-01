import { untrack } from 'svelte';
type CleanupFunction = () => void;

type CreateDropZoneOptions = {
	/**
	 * Whether to allow multiple files to be dropped.
	 * @default true
	 */
	multiple?: boolean;
	/**
	 * Callback for when files are dropped.
	 */
	onDrop?(files: File[] | null, event: DragEvent): void;
	/**
	 * Callback for when files are pasted.
	 */
	onPaste?(files: File[] | null, event: ClipboardEvent): void;
};

type CreateDropZoneReturn = {
	readonly isOver: boolean;
	files: File[] | null;
	cleanup: CleanupFunction;
};

/**
 * Creates a zone where files can be dropped.
 * Standalone migration from sv-use.
 */
export function createDropZone(
	target: () => HTMLElement | null | undefined,
	options: CreateDropZoneOptions
): CreateDropZoneReturn {
	// Inline utilities from sv-use
	const noop = () => {};

	const { multiple = true, onDrop = noop, onPaste = noop } = options;

	let cleanups: CleanupFunction[] = [];
	let counter = 0;

	const _target = $derived(target());
	let isOver = $state(false);
	let files = $state<File[] | null>(null);

	$effect(() => {
		const targetElement = _target;
		if (targetElement) {
			untrack(() => {
				const addListener = (
					event: 'dragenter' | 'dragover' | 'dragleave' | 'drop',
					handler: (event: DragEvent) => void
				) => {
					const listener = (e: DragEvent) => handler(e);
					targetElement.addEventListener(event, listener);
					return () => targetElement.removeEventListener(event, listener);
				};

				cleanups.push(
					addListener('dragenter', (event) => handleDragEvent(event, 'enter')),
					addListener('dragover', (event) => handleDragEvent(event, 'over')),
					addListener('dragleave', (event) => handleDragEvent(event, 'leave')),
					addListener('drop', (event) => handleDragEvent(event, 'drop'))
				);

				const pasteListener = (e: ClipboardEvent) => handlePasteEvent(e);
				targetElement.addEventListener('paste', pasteListener);
				cleanups.push(() => targetElement.removeEventListener('paste', pasteListener));
			});
		}

		return () => {
			cleanups.forEach((fn) => fn());
			cleanups = [];
		};
	});

	function handlePasteEvent(event: ClipboardEvent) {
		const clipboardFiles = event.clipboardData?.files;
		if (!clipboardFiles || clipboardFiles.length === 0) return;

		const list = Array.from(clipboardFiles);
		const currentFiles = list.length === 0 ? null : multiple ? list : [list[0]];

		if (currentFiles) {
			files = currentFiles;
			onPaste(currentFiles, event);
		}
	}

	function getFiles(event: DragEvent) {
		const list = Array.from(event.dataTransfer?.files ?? []);
		return list.length === 0 ? null : multiple ? list : [list[0]];
	}

	function handleDragEvent(event: DragEvent, eventType: 'enter' | 'over' | 'leave' | 'drop') {
		// Accept all file types
		event.preventDefault();
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'copy';
		}

		const currentFiles = getFiles(event);

		switch (eventType) {
			case 'enter':
				counter += 1;
				isOver = true;
				break;
			case 'over':
				break; // No onOver callback
			case 'leave':
				counter -= 1;
				if (counter === 0) {
					isOver = false;
				}
				break;
			case 'drop':
				counter = 0;
				isOver = false;
				files = currentFiles;
				onDrop(currentFiles, event);
				break;
		}
	}

	function cleanup() {
		cleanups.forEach((fn) => fn());
	}

	return {
		get files() {
			return files;
		},
		set files(v) {
			files = v;
		},
		get isOver() {
			return isOver;
		},
		cleanup
	};
}
