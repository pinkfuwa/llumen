import { writable } from 'svelte/store';
import { token } from './store';
import { onDestroy } from 'svelte';

let latestError = writable<{
	id: number;
	error: string;
	reason?: string;
} | null>(null);

export function dispatchError(errorMsg: string, reason?: string) {
	latestError.update((prev) => {
		const lastId = prev ? prev.id : 0;

		return {
			id: lastId + 1,
			error: errorMsg,
			reason
		};
	});
}

export function useError() {
	return latestError;
}

export function dismissError() {
	latestError.set(null);
}

export function initError() {
	// dispatch error
	const error = useError();
	const unsubscriber = error.subscribe((error) => {
		if (error?.error == 'malformed_token') token.set(undefined);
	});

	onDestroy(() => unsubscriber());
}
