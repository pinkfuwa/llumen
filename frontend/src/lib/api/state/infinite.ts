import { type Readable, type Writable, derived, get, writable } from 'svelte/store';
import { globalCache } from './cache';
import { CreateInternalQuery } from './internal';
import { onDestroy } from 'svelte';
import { Cleanups, nextCount } from './helper';

export interface Page<D> {
	fetch(): Promise<D[] | undefined>;
	nextPage(): Page<D> | undefined;
	/**
	 * insert at front, return new page(previous page) if overflow.
	 * @param data
	 */
	insertFront(data: D): Page<D> | undefined;
}

export interface InfiniteQueryEntry<D> {
	page: Page<D>;
	data: Writable<D[]>;
	target: Writable<HTMLElement | null>;
	revalidate: () => Promise<void>;
}

export interface InfiniteQueryOption<D> {
	key: string[];
	firstPage: Page<D>;
}

export interface InfiniteQueryResult<D> {
	data: Readable<InfiniteQueryEntry<D>[]>;
}

export function CreateInfiniteQuery<D>(option: InfiniteQueryOption<D>): InfiniteQueryResult<D> {
	const { key, firstPage } = option;
	const data = globalCache.getOrExecute<Array<InfiniteQueryEntry<D>>>(['inf', ...key], () => []);
	data.set([]);

	const cleanup = new Cleanups();

	let totalPage = 0;
	function addPage(page: Page<D>) {
		const target = writable<HTMLElement | null>(null);

		let stablizedNext = false;

		function applyNext(data: D[] | undefined) {
			if (stablizedNext || data == undefined) return;
			const nextPage = page.nextPage();
			if (nextPage) {
				stablizedNext = true;
				addPage(nextPage);
			}
		}

		const query = CreateInternalQuery<D[]>({
			fetcher: () => page.fetch(),
			key: [...key, 'page', (totalPage++).toString()],
			target,
			onSuccess: applyNext,
			initialData: [],
			staleTime: 30000,
			cleanupCallback: (callback) => cleanup.add(callback)
		});

		data.update((x) => {
			x.push({
				page: page,
				data: query.data as Writable<D[]>,
				target,
				revalidate: query.revalidate
			});
			return x;
		});
	}

	addPage(firstPage);

	return { data };
}

export function PushFrontInfiniteQueryData<D>(key: string[], newData: D) {
	const data = globalCache.getOrExecute<Array<InfiniteQueryEntry<D>>>(['inf', ...key], () => []);

	const cleanup = new Cleanups();

	data.update((x) => {
		let first = x[0];
		let newPage = first.page.insertFront(newData);
		const target = writable<HTMLElement | null>(null);
		if (newPage) {
			const query = CreateInternalQuery<D[]>({
				fetcher: () => newPage.fetch(),
				key: [...key, nextCount().toString()],
				target,
				initialData: [],
				cleanupCallback: (callback) => cleanup.add(callback)
			});
			x.unshift({
				page: newPage,
				data: query.data as Writable<D[]>,
				target,
				revalidate: query.revalidate
			});
		} else {
			first.data.update((data) => {
				data.unshift(newData);
				return data;
			});
		}

		return x;
	});
}
