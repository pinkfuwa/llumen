import { type Readable, type Writable, derived, get, readable, writable } from 'svelte/store';
import { CreateInternalQuery } from './internal';
import { onDestroy } from 'svelte';
import { globalCache } from './cache';

export interface Fetcher<D extends { id: number }> {
	range(startId: number, endId: number): Promise<D[] | undefined>;
	forward(limit: number, id?: number): Promise<D[] | undefined>;
	backward(limit: number, id: number): Promise<D[] | undefined>;
}

interface Page<D> {
	no: number;
	startId?: number;
	endId?: number;
	data: Writable<D[]>;
	target: Writable<HTMLElement | null>;
	revalidate: () => void;
}

class Pages<D extends { id: number }> {
	private maxSize = 16;
	private fetcher: Fetcher<D>;
	pages: Writable<Page<D>[]> = writable([]);
	constructor(fetcher: Fetcher<D>, id?: number | null) {
		this.fetcher = fetcher;
		if (id !== undefined) {
			const startId = id === null ? undefined : id;
			this.pages.set([
				{
					no: 0,
					data: writable([]),
					startId,
					target: writable(null),
					revalidate: () => console.error('Unreachable!')
				}
			]);
		}
	}
	private activatePage(page: Page<D>, cleanupCallback: (d: () => void) => void) {
		// author's note: duplicated code, but it's more readable!
		const addForwardPage = () => {
			this.pages.update((x) => {
				const last = x.at(-1)!;

				const newPage = {
					no: last.no + 1,
					data: writable([]),
					startId: last.endId! + 1, // TODO: if you want to use ascending, change this
					target: writable(null),
					revalidate: () => console.error('Unreachable!')
				};

				x.push(newPage);
				this.activatePage(newPage, cleanupCallback);

				return x;
			});
		};

		const addBackwardPage = () => {
			this.pages.update((x) => {
				const first = x[0];

				const newPage = {
					no: first.no - 1,
					data: writable([]),
					endId: first.startId! + 1, // TODO: if you want to use ascending, change this
					target: writable(null),
					revalidate: () => console.error('Unreachable!')
				};

				x.unshift(newPage);
				this.activatePage(newPage, cleanupCallback);
				return x;
			});
		};

		const query = CreateInternalQuery<D[]>({
			fetcher: () => {
				if (page.endId != undefined && page.startId != undefined)
					return this.fetcher.range(page.startId, page.endId);
				if (page.endId != undefined) return this.fetcher.backward(this.maxSize, page.endId);
				return this.fetcher.forward(this.maxSize, page.startId);
			},
			key: page.data,
			target: page.target,
			onSuccess: (data: D[] | undefined) => {
				if (data == undefined || data.length == 0) return;

				// first page special case: always add a backward page
				if (page.startId == undefined && page.endId == undefined) {
					page.startId = data[0].id;
					addBackwardPage();
				}

				// no > 0 and first page
				if (page.endId == undefined && data.length >= this.maxSize) {
					page.endId = data.at(-1)!.id;
					addForwardPage();
				}

				// no < 0
				if (page.startId == undefined && data.length >= this.maxSize) {
					page.startId = data[0].id;
					addBackwardPage();
				}
			},
			initialData: [],
			staleTime: 30000,
			cleanupCallback,
			revalidateOnFocus: true
		});
		page.revalidate = query.revalidate;
	}
	/**
	 * start background fetching
	 */
	public activate() {
		const callbacks: Array<() => void> = [];
		onDestroy(() => callbacks.forEach((x) => x()));

		get(this.pages).forEach((page) =>
			this.activatePage(page, (callback) => callbacks.push(callback))
		);
	}
	public insertData(data: D) {
		const pages = get(this.pages);
		if (pages.length == 0) return;

		pages[0].data.update((x) => {
			x.unshift(data);
			return x;
		});
	}
	/**
	 * Remove contineous data/page from beginning of pages where predicate return true
	 * @param predicate
	 */
	public removeData(predicate: (data: D) => boolean) {
		this.pages.update((x) => {
			const numberOfRemovalPage = x.findLastIndex((x) => get(x.data).some(predicate));
			if (numberOfRemovalPage == -1) return x;
			x.splice(0, numberOfRemovalPage);

			x[0].data.update((x) => x.filter((x) => !predicate(x)));
			x[0].startId = undefined;

			return x;
		});
	}
	/**
	 * revalidate pages where predicate return true
	 */
	public revalidate(predicate: (data: D) => boolean) {
		get(this.pages).forEach((page) => {
			if (get(page.data).some(predicate)) page.revalidate();
		});
	}
	public revalidateAll() {
		get(this.pages).forEach((page) => page.revalidate());
	}
}

export interface InfiniteQueryOption<D extends { id: number }> {
	fetcher: Fetcher<D>;
	key: string[];
	id?: number;
}

export interface PageEntry<D> {
	target: Writable<HTMLElement | null>;
	data: Readable<D[]>;
	no: number;
	revalidate: () => void;
}

export interface InfiniteQueryResult<D extends { id: number }> {
	data: Readable<Array<PageEntry<D>>>;
}

export function CreateInfiniteQuery<D extends { id: number }>(
	option: InfiniteQueryOption<D>
): InfiniteQueryResult<D> {
	let { key, fetcher, id } = option;

	const pageStore = globalCache.getOrExecute(
		key,
		() => new Pages<D>(fetcher, id == undefined ? null : id)
	);

	const pages = get(pageStore);
	pages.activate();

	return { data: pages.pages };
}

export function SetInfiniteQueryData<D extends { id: number }>(option: { data: D; key: string[] }) {
	let { key, data } = option;

	const pageStore = globalCache.get<Pages<D>>(key);

	const pages = get(pageStore);
	if (pages) pages.insertData(data);
}

export function RemoveInfiniteQueryData<D extends { id: number }>(option: {
	predicate: (data: D) => boolean;
	key: string[];
}) {
	let { key, predicate } = option;

	const pageStore = globalCache.get<Pages<D>>(key);

	const pages = get(pageStore);
	if (pages) pages.removeData(predicate);
}

export function RevalidateInfiniteQueryData<D extends { id: number }>(option: {
	predicate?: (data: D) => boolean;
	key: string[];
}) {
	let { key, predicate } = option;

	const pageStore = globalCache.get<Pages<D>>(key);

	const pages = get(pageStore);
	if (pages) {
		if (predicate == undefined) pages.revalidateAll();
		else pages.revalidate(predicate);
	}
}
