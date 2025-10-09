import { untrack } from 'svelte';
type MaybeGetter<T> = T | (() => T);
type CleanupFunction = () => void;

type CreateDropZoneOptions = {
	/**
	 * The allowed data types in the format `xxx/xxx`.
	 * Supports `*` and `xxx/*` wildcards.
	 * @default '*'
	 */
	allowedDataTypes?: MaybeGetter<string> | MaybeGetter<string[]> | ((types: string[]) => boolean);
	/**
	 * Whether to allow multiple files to be dropped.
	 * @default true
	 */
	multiple?: boolean;
	/**
	 * Callback for when files are dropped.
	 */
	onDrop?(files: File[] | null, event: DragEvent): void;
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
	target: MaybeGetter<HTMLElement | null | undefined>,
	options: CreateDropZoneOptions = {}
): CreateDropZoneReturn {
	// Inline utilities from sv-use
	const noop = () => {};
	const normalizeValue = <T>(value: MaybeGetter<T>): T =>
		typeof value === 'function' ? (value as () => T)() : value;
	const toArray = <T>(value: T | T[]): T[] => (Array.isArray(value) ? value : [value]);

	const { allowedDataTypes = '*', multiple = true, onDrop = noop } = options;

	let cleanups: CleanupFunction[] = [];
	let counter = 0;
	let isValid = true;

	const _target = $derived(normalizeValue(target));
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
			});
		}

		return () => {
			cleanups.forEach((fn) => fn());
			cleanups = [];
		};
	});

	function getFiles(event: DragEvent) {
		const list = Array.from(event.dataTransfer?.files ?? []);
		return list.length === 0 ? null : multiple ? list : [list[0]];
	}

	function checkDataTypes(types: string[]): boolean {
		if (types.length === 0) return false;
		if (typeof allowedDataTypes === 'function' && allowedDataTypes.length > 0) {
			return allowedDataTypes(types) as boolean;
		}
		if (allowedDataTypes === '*') return true;

		const _allowedTypes = allowedDataTypes as MaybeGetter<string> | MaybeGetter<string[]>;
		const allowedArray = toArray(normalizeValue(_allowedTypes));

		return types.every((type) => {
			return allowedArray.some((allowedType) => {
				const [prefix] = allowedType.split('/');
				if (allowedType.split('/')[1] === '*') {
					return type.startsWith(prefix);
				}
				return type === allowedType;
			});
		});
	}

	function checkValidity(items: DataTransferItemList) {
		const types = Array.from(items ?? []).map((item) => item.type);

		const dataTypesValid = checkDataTypes(types);
		const multipleFilesValid = multiple || items.length <= 1;

		return dataTypesValid && multipleFilesValid;
	}

	function isSafari() {
		return /^(?:(?!chrome|android).)*safari/i.test(navigator.userAgent) && !('chrome' in window);
	}

	function handleDragEvent(event: DragEvent, eventType: 'enter' | 'over' | 'leave' | 'drop') {
		const dataTransferItemList = event.dataTransfer?.items;
		isValid = (dataTransferItemList && checkValidity(dataTransferItemList)) ?? false;

		// preventDefaultForUnhandled is hardcoded to false, so no always-prevent here

		if (!isSafari() && !isValid) {
			if (event.dataTransfer) {
				event.dataTransfer.dropEffect = 'none';
			}
			return;
		}

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
				if (isValid) {
					files = currentFiles;
					onDrop(currentFiles, event);
				}
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
