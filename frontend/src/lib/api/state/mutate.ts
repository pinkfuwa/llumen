import { type Readable, get, readable, writable } from 'svelte/store';
import { useToken } from '$lib/store';

export interface mutationResult<P, D> {
	mutate: (param: P, callback?: (data: D) => void) => Promise<D | undefined>;
	isPending: Readable<boolean>;
	isError: Readable<boolean>;
}

export interface useMutateOption<P, D> {
	mutator: (param: P, token?: string) => Promise<D>;
	onSuccess?: (data: D) => void;
}

export function useMutate<P, D>(option: useMutateOption<P, D>): mutationResult<P, D> {
	let token = useToken();

	const { mutator, onSuccess } = option;

	const isPendingWritable = writable(false);
	const isErrorWritable = writable(false);

	const isPending = readable(false, (set) => {
		return isPendingWritable.subscribe(set);
	});
	const isError = readable(false, (set) => {
		return isErrorWritable.subscribe(set);
	});

	async function mutate(param: P, callback?: (data: D) => void) {
		isPendingWritable.set(true);
		isErrorWritable.set(false);

		try {
			const result = await mutator(param, get(token));

			if (callback) callback(result);
			if (onSuccess) onSuccess(result);

			return result;
		} catch (err) {
			isErrorWritable.set(true);
			isPendingWritable.set(false);

			console.warn('error running mutation', err);
			return undefined;
		} finally {
			isPendingWritable.set(false);
		}
	}

	return {
		mutate,
		isPending,
		isError
	};
}
