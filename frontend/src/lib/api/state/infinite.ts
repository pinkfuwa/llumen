import { type Readable, type Writable, get, writable } from 'svelte/store';
import { globalCache } from './cache';
import { CreateInternalQuery } from './internal';
import { Cleanups } from './helper';

export interface Page<D> {
	fetch(): Promise<D[] | undefined>;
	nextPage(): Page<D> | undefined;
}

export class InfiniteQueryEntries<D> {
	key: string[];
	constructor(key: string[], page: Page<D>) {
		this.key = key;

		if (this.getEntries().length == 0) this.addForwardPage(page);
	}
	incrementCount(): number {
		const page = globalCache.getOr([...this.key, 'pageNo'], [0, 1] as [number, number]);
		page.update(([x, y]) => [x, y + 1]);
		return get(page)[1];
	}
	addForwardPage(page: Page<D>): InfiniteQueryEntry<D> {
		const no = this.incrementCount();
		const entry = new InfiniteQueryEntry(page, no, this.key);
		this.pushEntries(entry);
		return entry;
	}
	pushEntries(entry: InfiniteQueryEntry<D>) {
		const entries = globalCache.get(this.key)! as Writable<Array<InfiniteQueryEntry<D>>>;

		entries.update((x) => [...x, entry]);
	}
	getEntries() {
		const entries = globalCache.get(this.key)! as Writable<Array<InfiniteQueryEntry<D>>>;
		return get(entries);
	}
	activate(cleanup: Cleanups) {
		this.getEntries().forEach((entry) => {
			const addPage = (page: Page<D>) => {
				const entry = this.addForwardPage(page);
				entry.activate(addPage, cleanup);
			};
			entry.activate(addPage, cleanup);
		});
	}
}

export class InfiniteQueryEntry<D> {
	page: Page<D>;
	data: Writable<D[]> = writable([]);
	target: Writable<HTMLElement | null> = writable(null);
	revalidate: () => Promise<void> = () => Promise.resolve();
	hasNext: boolean;
	parentKey: string[];
	key: string[];

	constructor(page: Page<D>, no: number, key: string[], hasNext = false) {
		this.parentKey = key;
		this.key = [...key, no.toString()];

		this.page = page;
		this.hasNext = hasNext;
	}
	heatParentKey() {
		globalCache.get(this.parentKey);
	}
	checkNextPage(addPage: (page: Page<D>) => void) {
		if (this.hasNext) return;

		const nextPage = this.page.nextPage();
		if (nextPage) {
			this.hasNext = true;
			addPage(nextPage);
		}
	}
	activate(addPage: (page: Page<D>) => void, cleanup: Cleanups) {
		const query = CreateInternalQuery<D[]>({
			fetcher: () => this.page.fetch(),
			key: this.key,
			target: this.target,
			onSuccess: (data: D[] | undefined) => {
				this.heatParentKey();
				if (data != undefined) this.checkNextPage(addPage);
			},
			initialData: [],
			staleTime: 30000,
			cleanupCallback: (callback) => cleanup.add(callback)
		});
		this.data = query.data as Writable<D[]>;
		this.revalidate = query.revalidate;
	}
}

export interface InfiniteQueryOption<D> {
	key: string[];
	firstPage: Page<D>;
}

export interface InfiniteQueryResult<D> {
	data: Readable<Array<InfiniteQueryEntry<D>>>;
}

export function CreateInfiniteQuery<D>(option: InfiniteQueryOption<D>): InfiniteQueryResult<D> {
	const { key, firstPage } = option;

	const data = globalCache.getOr(key, []) as Writable<Array<InfiniteQueryEntry<D>>>;

	const entries = new InfiniteQueryEntries<D>(key, firstPage);
	entries.activate(new Cleanups());

	return { data };
}

export class KeysetPage<D extends { id: number }> implements Page<D> {
	private max_size = 2;
	private fetcher: (limit: number, id?: number) => Promise<D[] | undefined>;
	private ids: number[] = [];
	constructor(fetcher: (limit: number, id?: number) => Promise<D[] | undefined>) {
		this.fetcher = fetcher;
	}
	async fetch(): Promise<D[] | undefined> {
		const list = await this.fetcher(
			this.max_size,
			this.ids.length != 0 ? this.ids.at(0)! + 1 : undefined
		);
		if (!list) return;

		if (list.length != 0) this.ids = list.map((x) => x.id);
		return list;
	}
	nextPage(): Page<D> | undefined {
		if (this.ids.length >= this.max_size) {
			const page = new KeysetPage(this.fetcher);
			page.ids = [this.ids.at(-1)! - 1];
			return page;
		}
	}
}
