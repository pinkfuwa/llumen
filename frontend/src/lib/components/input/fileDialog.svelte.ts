// Inline BROWSER if no esm-env: const BROWSER = typeof document !== 'undefined' && typeof window !== 'undefined';

type CreateFileDialogOptions = {
	/**
	 * MIME types to accept (e.g., 'image/*' or '*.png').
	 * @default '*'
	 */
	accept?: string;
	/**
	 * Whether to allow multiple files.
	 * @default false
	 */
	multiple?: boolean;
	/**
	 * Triggers when files are selected.
	 */
	onChange?: (files: File[]) => void;
};

type CreateFileDialogReturn = {
	/**
	 * A list of selected files (reactive).
	 * @readonly
	 */
	readonly files: File[];
	/** Opens the file dialog. */
	open: () => void;
	/** Resets the file dialog. */
	reset: () => void;
	/** Cleans up the input node and event listeners. */
	cleanup: () => void;
};

/**
 * Creates a file dialog to interact with programmatically.
 * Standalone migration from sv-use.
 */
export function createFileDialog(options: CreateFileDialogOptions = {}): CreateFileDialogReturn {
	const { accept = '*', multiple = false, onChange = () => {} } = options;

	let _files = $state<File[]>([]);
	let _input: HTMLInputElement | null = null;

	const cleanups: Array<() => void> = [];

	_input = document.createElement('input');
	_input.type = 'file';
	_input.accept = accept;
	_input.multiple = multiple;

	// Inline handleEventListener: Add listeners with manual cleanup
	const addListener = (event: string, handler: (event: Event) => void) => {
		const listener = (e: Event) => handler(e);
		_input!.addEventListener(event, listener);
		return () => _input!.removeEventListener(event, listener);
	};

	cleanups.push(
		addListener('change', (event) => {
			const input = event.currentTarget as HTMLInputElement;
			_files = Array.from(input.files ?? []);
			onChange(_files);
		})
		// 'cancel' listener omitted as onCancel is unused
	);

	function open() {
		_input?.click();
	}

	function reset() {
		_files = [];
		_input && (_input.value = '');
	}

	function cleanup() {
		cleanups.forEach((fn) => fn());
		_input?.remove();
		_input = null;
	}

	// Optional: Run cleanup on disposal if in a component, but manual call recommended for standalone
	// e.g., You can wrap in $effect(() => cleanup()) if needed, but avoided here for flexibility

	return {
		get files() {
			return _files;
		},
		open,
		reset,
		cleanup
	};
}
