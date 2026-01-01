export { createMutation, createRawMutation } from './mutate.svelte';
export type {
	MutationResult,
	RawMutationResult,
	CreateMutationOption,
	CreateRawMutateOption
} from './mutate.svelte';

export { createQueryEffect } from './query.svelte';
export type { QueryEffectOption } from './query.svelte';

export {
	createInfiniteQueryEffect,
	insertInfiniteQueryData,
	updateInfiniteQueryDataById,
	removeInfiniteQueryData,
	getInfiniteQueryData
} from './infinite.svelte';
export type { InfiniteQueryEffectOption, PageState, Fetcher } from './infinite.svelte';

export { CreateMockMutation, CreateMockQuery } from './mock.svelte';
export { getError } from './errorHandle';
