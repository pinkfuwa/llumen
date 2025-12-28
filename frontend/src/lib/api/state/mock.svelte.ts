import type { MutationResult } from './mutate.svelte';

export async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export function CreateMockMutation<P, D>(result: D): MutationResult<P, D> {
	let isPending = $state(false);
	let isError = $state(false);

	return {
		mutate: async (param: P, callback?: (data: D) => void) => {
			isPending = true;
			await sleep(10);
			isPending = false;
			if (callback) callback(result);
			return result;
		},
		isError: () => isError,
		isPending: () => isPending
	};
}

export function CreateMockQuery<D>(result: D): {
	revalidate: () => Promise<void>;
	isLoading: () => boolean;
	getData: () => D | undefined;
} {
	let data = $state<D | undefined>(undefined);
	let isLoading = $state(false);

	(async () => {
		isLoading = true;
		await sleep(10);
		data = result;
		isLoading = false;
	})();

	return {
		getData: () => data,
		revalidate: () => Promise.resolve(),
		isLoading: () => isLoading
	};
}
