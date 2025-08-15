import { writable } from 'svelte/store';

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
