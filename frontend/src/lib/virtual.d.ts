import 'svelte/elements';

declare module 'svelte/elements' {
	export interface DOMAttributes<T extends HTMLTextAreaElemen> {
		virtualkeyboardpolicy?: 'auto' | 'manual' | (string & {});
	}
}

export {};

declare global {
	interface Navigator {
		virtualKeyboard: VirtualKeyboard;
	}

	interface VirtualKeyboard {
		overlaysContent: boolean;
		boundingRect: DOMRect;
		show(): Promise<void>;
		hide(): Promise<void>;
		addEventListener(
			type: 'geometrychange',
			listener: (event: VirtualKeyboardGeometryChangeEvent) => void,
			options?: boolean | AddEventListenerOptions
		): void;
		removeEventListener(
			type: 'geometrychange',
			listener: (event: VirtualKeyboardGeometryChangeEvent) => void,
			options?: boolean | EventListenerOptions
		): void;
	}

	interface VirtualKeyboardGeometryChangeEvent extends Event {
		target: VirtualKeyboard;
	}
}
