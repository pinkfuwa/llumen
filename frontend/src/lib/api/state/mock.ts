import { writable } from 'svelte/store';
import type { MutationResult } from './mutate';
import type { QueryResult } from './query';

export async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export function CreateMockMutation<P, D>(
	result: D,
	oncall?: (param: P) => void
): MutationResult<P, D> {
	const isPending = writable(false);
	const isError = writable(false);

	return {
		mutate: async (param: P, callback?: (data: D) => void) => {
			if (oncall) oncall(param);
			isPending.set(true);
			await sleep(10);
			isPending.set(false);
			if (callback) callback(result);
			return result;
		},
		isError,
		isPending
	};
}

export function CreateMockQuery<D>(result: D): QueryResult<D> {
	let data = writable<D | undefined>(undefined);
	let isLoading = writable(false);

	(async () => {
		isLoading.set(true);
		await sleep(10);
		data.set(result);
		isLoading.set(false);
	})();

	return {
		data,
		revalidate: () => Promise.resolve(),
		isLoading
	};
}
