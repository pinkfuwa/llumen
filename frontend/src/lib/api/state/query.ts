import { type Readable, type Writable } from 'svelte/store';
import { CreateInternalQuery } from './internal';
import { APIFetch } from './errorHandle';

export interface QueryResult<T> {
	data: Writable<T | undefined>;
	revalidate: () => Promise<void>;
	isLoading: Readable<boolean>;
}

export interface QueryOption<P, D> {
	path: string | (() => string);
	body?: P | (() => P);
	method: 'POST' | 'GET' | 'PUT' | 'UPDATE';
	key?: string[];
	staleTime?: number;
	target?: Readable<HTMLElement | null>;
	revalidateOnFocus?: boolean | 'force';
}

export function CreateQuery<P, D>(option: QueryOption<P, D>): QueryResult<D> {
	let { target, key, staleTime, revalidateOnFocus, path, body, method } = option;

	const getPath = () => (path instanceof Function ? path() : path);
	const getBody = () => (body instanceof Function ? body() : body);

	const fetcher = async () => APIFetch<D>(getPath(), getBody(), method);

	if (!staleTime) staleTime = 30000;

	return CreateInternalQuery({
		target,
		key: key ? key : ['query', getPath()],
		staleTime,
		revalidateOnFocus,
		fetcher
	});
}
