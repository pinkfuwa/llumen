import { type Readable, get, readable, writable } from 'svelte/store';
import { APIFetch } from './errorHandle';

export interface RawMutationResult<P, D> {
	mutate: (param: P, callback?: (data: D) => void) => Promise<D | undefined>;
	isPending: Readable<boolean>;
	isError: Readable<boolean>;
}

export interface CreateRawMutateOption<P, D> {
	mutator: (param: P) => Promise<D | undefined>;
	onSuccess?: (data: D, param: P) => void;
}

export function CreateRawMutation<P, D>(
	option: CreateRawMutateOption<P, D>
): RawMutationResult<P, D> {
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

		try {
			const result = await mutator(param);

			if (result) {
				if (callback) callback(result);
				if (onSuccess) onSuccess(result, param);

				isErrorWritable.set(false);

				return result;
			} else {
				isErrorWritable.set(true);
			}
		} catch (err) {
			isErrorWritable.set(true);

			console.warn('error running mutation', err);
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

export interface MutationResult<P, D> {
	mutate: (param: P, callback?: (data: D) => void) => Promise<D | undefined>;
	isPending: Readable<boolean>;
	isError: Readable<boolean>;
}

export interface CreateMutateOption<P, D> {
	onSuccess?: (data: D, param: P) => void;
	path: string | (() => string);
	method?: 'POST' | 'GET' | 'PUT' | 'UPDATE';
}

export function CreateMutation<P, D>(option: CreateMutateOption<P, D>): MutationResult<P, D> {
	const { path, method, onSuccess } = option;

	const getPath = typeof path === 'function' ? path : () => path;

	const mutator = (param: P) => APIFetch<D>(getPath(), param, method);

	return CreateRawMutation({
		mutator,
		onSuccess
	});
}
