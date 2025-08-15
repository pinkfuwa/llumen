import { type Readable, type Writable, derived, get, writable } from 'svelte/store';
import { globalCache } from './cache';
import { CreateInternalQuery } from './internal';

export interface Page<D> {
	fetch(): Promise<D[]>;
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

	function addPage(page: Page<D>) {
		let stablizedNext = false;
		const target = writable(null);
		const query = CreateInternalQuery<D[]>({
			fetcher: () => page.fetch(),
			key,
			target,
			onSuccess(data) {
				if (stablizedNext || data == undefined) return;
				const nextPage = page.nextPage();
				if (nextPage) {
					stablizedNext = true;
					addPage(nextPage);
				}
			},
			initialData: []
		});

		query.data.subscribe((data) => {});

		data.update((x) => {
			x.push({
				page: page,
				data: query.data as Writable<D[]>,
				target
			});
			return x;
		});
	}

	addPage(firstPage);

	return { data };
}

export function PushFrontInfiniteQueryData<D>(key: string[], newData: D) {
	const data = globalCache.getOrExecute<Array<InfiniteQueryEntry<D>>>(['inf', ...key], () => []);

	data.update((x) => {
		let first = x[0];
		let newPage = first.page.insertFront(newData);
		const target = writable(null);
		if (newPage) {
			const query = CreateInternalQuery<D[]>({
				fetcher: () => newPage.fetch(),
				key,
				target,
				initialData: []
			});
			x.unshift({
				page: newPage,
				data: query.data as Writable<D[]>,
				target
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
// export interface RecursiveQueryResult<D, IS> {
// 	data: Writable<D | undefined>;
// 	revalidate: () => Promise<D>;
// 	isLoading: Readable<boolean>;
// 	nextParam: Readable<IS | undefined>;
// }

// export interface RecursiveQueryOption<D, IS, S = IS> {
// 	initialParam: IS;
// 	nextParam: (lastPage: D) => IS;
// 	genParam: (param: IS, page: D) => S | undefined;
// 	fetcher: (param: IS | S, token?: string) => Promise<D>;
// 	key?: string[];
// 	staleTime?: number;
// 	/**
// 	 * fetch next page/refetch only when target element is visible
// 	 * @returns target element
// 	 */
// 	target?: () => HTMLElement | null | undefined;
// 	revalidateOnFocus?: boolean;
// }

// export function CreateRecursiveQuery<D, IS, S = IS>(
// 	option: RecursiveQueryOption<D, IS, S>
// ): RecursiveQueryResult<D, IS> {
// 	const {
// 		initialParam,
// 		fetcher,
// 		staleTime,
// 		target,
// 		genParam,
// 		nextParam: genNextParam,
// 		revalidateOnFocus,
// 		key
// 	} = option;

// 	let currentParam: S | undefined;
// 	const queryResult = CreateQuery({
// 		param: () => (currentParam == undefined ? initialParam : currentParam),
// 		fetcher: async (param, token) => {
// 			let result = await fetcher(param, token);
// 			if (currentParam == undefined) currentParam = genParam(param as IS, result);
// 			return result;
// 		},
// 		staleTime,
// 		target,
// 		revalidateOnFocus,
// 		key
// 	});

// 	let canFetchNext = writable(target == undefined);

// 	if (target) {
// 		observeIntersection(
// 			target,
// 			([entry]) => {
// 				const isVisible = entry?.isIntersecting || false;
// 				canFetchNext.update((x) => x || isVisible);
// 			},
// 			{}
// 		);
// 	}

// 	let savedNextParam: Writable<IS | undefined> = writable(undefined);

// 	const nextParam = derived(
// 		[queryResult.data, canFetchNext, savedNextParam],
// 		([$page, $canFetchNext, $savedNextParam]: [D | undefined, boolean, IS | undefined]) => {
// 			if ($savedNextParam) return $savedNextParam;
// 			if ($canFetchNext && $page != undefined) {
// 				const nextParam = genNextParam($page);
// 				savedNextParam.set(nextParam);
// 				return nextParam;
// 			}
// 		}
// 	);

// 	return { nextParam, ...queryResult };
// }
