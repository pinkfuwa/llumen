import { APIFetch } from './errorHandle';

export interface MutationResult<P, D> {
	mutate: (param: P, callback?: (data: D) => void) => Promise<D | undefined>;
	isPending: () => boolean;
	isError: () => boolean;
}

export interface CreateMutationOption<P, D> {
	onSuccess?: (data: D, param: P) => void;
	path: string | (() => string);
	method?: 'POST' | 'GET' | 'PUT' | 'UPDATE';
}

/**
 * Creates a mutation function that sends data to the server.
 * Must be called during component initialization.
 */
export function createMutation<P, D>(option: CreateMutationOption<P, D>): MutationResult<P, D> {
	const { path, method, onSuccess } = option;

	const getPath = typeof path === 'function' ? path : () => path;

	let isPending = $state(false);
	let isError = $state(false);

	async function mutate(param: P, callback?: (data: D) => void) {
		isPending = true;

		try {
			const result = await APIFetch<D>(getPath(), param, method);

			if (result) {
				if (callback) callback(result);
				if (onSuccess) onSuccess(result, param);

				isError = false;

				return result;
			} else {
				isError = true;
			}
		} catch (err) {
			isError = true;
			console.warn('error running mutation', err);
		} finally {
			isPending = false;
		}
	}

	return {
		mutate,
		isPending: () => isPending,
		isError: () => isError
	};
}

export interface RawMutationResult<P, D> {
	mutate: (param: P, callback?: (data: D) => void) => Promise<D | undefined>;
	isPending: () => boolean;
	isError: () => boolean;
}

export interface CreateRawMutateOption<P, D> {
	mutator: (param: P) => Promise<D | undefined>;
	onSuccess?: (data: D, param: P) => void;
}

/**
 * Creates a mutation with a custom mutator function.
 * Must be called during component initialization.
 */
export function createRawMutation<P, D>(
	option: CreateRawMutateOption<P, D>
): RawMutationResult<P, D> {
	const { mutator, onSuccess } = option;

	let isPending = $state(false);
	let isError = $state(false);

	async function mutate(param: P, callback?: (data: D) => void) {
		isPending = true;

		try {
			const result = await mutator(param);

			if (result) {
				if (callback) callback(result);
				if (onSuccess) onSuccess(result, param);

				isError = false;

				return result;
			} else {
				isError = true;
			}
		} catch (err) {
			isError = true;

			console.warn('error running mutation', err);
		} finally {
			isPending = false;
		}
	}

	return {
		mutate,
		isPending: () => isPending,
		isError: () => isError
	};
}
