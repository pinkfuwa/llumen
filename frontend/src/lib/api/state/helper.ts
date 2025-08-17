import { onDestroy } from 'svelte';
import type { Readable } from 'svelte/store';

export function isElementInViewport(element: HTMLElement) {
	var rect = element.getBoundingClientRect();

	return (
		rect.top >= 0 &&
		rect.left >= 0 &&
		rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
		rect.right <= (window.innerWidth || document.documentElement.clientWidth)
	);
}

let count = 0;
export function nextCount() {
	return count++;
}
/**
 * helper function that execute when the storable satify the predicate.
 * @param s
 * @param predicate
 * @returns cleanup function
 */
export function once<T>(
	s: Readable<T>,
	predicate: (d: T) => boolean,
	f: (d: T) => void
): () => void {
	let hasRun = false;
	return s.subscribe((d) => {
		if (!hasRun && predicate(d)) {
			hasRun = true;
			f(d);
		}
	});
}

export class Cleanups {
	callbacks: Array<() => void> = [];
	constructor() {
		onDestroy(() => {
			this.callbacks.forEach((x) => x());
		});
	}
	add(callback: () => void) {
		this.callbacks.push(callback);
	}
}
