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
	count = count + 1;
	return count;
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

type VisibilityStateListenierId = number;
let visibilityListenerCount = 0;
let visibilityListeners = new Map<VisibilityStateListenierId, () => void>();

let visibilityHandlerAttached = false;

function runVisibilityListeners() {
	for (const cb of visibilityListeners.values()) {
		try {
			cb();
		} catch (err) {
			console.error('visibility listener error', err);
		}
	}
}

export function addVisibliityListener(callback: () => void): VisibilityStateListenierId {
	visibilityListenerCount += 1;
	const id = visibilityListenerCount;
	visibilityListeners.set(id, callback);

	if (!visibilityHandlerAttached) {
		// Attach the global handler once
		document.addEventListener('visibilitychange', runVisibilityListeners);
		visibilityHandlerAttached = true;
	}

	return id;
}

export function clearVisibilityListener(id: VisibilityStateListenierId) {
	visibilityListeners.delete(id);

	// If no more listeners, detach the global handler to avoid unnecessary work
	if (visibilityListeners.size === 0 && visibilityHandlerAttached) {
		document.removeEventListener('visibilitychange', runVisibilityListeners);
		visibilityHandlerAttached = false;
	}
}
