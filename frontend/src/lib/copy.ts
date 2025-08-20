import { writable } from 'svelte/store';

export const copyCounter = writable(0);

export function copy(text: string) {
	navigator.clipboard.writeText(text);
	copyCounter.update((x) => x + 1);
}
