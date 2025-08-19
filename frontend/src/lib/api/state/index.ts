import { globalCache } from './cache';

export { CreateMutation, CreateRawMutation } from './mutate';
export type {
	MutationResult as CreateMutationResult,
	CreateRawMutateOption,
	RawMutationResult
} from './mutate';
export { CreateQuery } from './query';
export type { QueryOption, QueryResult } from './query';
export { CreateInfiniteQuery, RemoveInfiniteQueryData, SetInfiniteQueryData } from './infinite';
export type { InfiniteQueryResult, InfiniteQueryOption, Fetcher, PageEntry } from './infinite';
export { CreateMockMutation, CreateMockQuery } from './mock';

export function clearCache() {
	globalCache.clear();
}
