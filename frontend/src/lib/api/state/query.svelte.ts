import { type Readable, type Writable, get, derived, writable, readable } from 'svelte/store';
import { useSWR } from 'sswr';
import { observeIntersection } from '@sv-use/core';
import { useToken } from '$lib/store';

const defaultStaleTime = 30000;

export interface QueryResult<T> {
	data: Writable<T | undefined>;
	revalidate: () => Promise<T>;
	isLoading: Readable<boolean>;
}

export interface QueryOption<P, D> {
	param: () => P;
	fetcher: (params: P, token?: string) => Promise<D>;
	key?: string[];
	staleTime?: number;
	/**
	 * refetch only when targetElementIsVisible
	 * @returns target element
	 */
	target?: () => HTMLElement | null | undefined;
	revalidateOnFocus?: boolean;
}

export function CreateQuery<P, D>(option: QueryOption<P, D>): QueryResult<D> {
	let token = useToken();
	let { target, key, param, fetcher, staleTime, revalidateOnFocus } = option;

	const { data, revalidate, isLoading } = useSWR(
		() =>
			JSON.stringify({
				key,
				param: param(),
				token: get(token)
			}),
		{
			fetcher: async (encoded: string) => {
				const { token, param } = JSON.parse(encoded);
				const execute = () => (token == '' ? fetcher(param) : fetcher(param, token));
				try {
					return await execute();
				} catch {
					return execute();
				}
			},
			revalidateOnFocus: revalidateOnFocus || false,
			revalidateOnReconnect: true
		}
	);

	let timeoutId: ReturnType<typeof setTimeout> | undefined;
	function setLoop(): ReturnType<typeof setTimeout> {
		return setTimeout(async () => {
			await revalidate();
			timeoutId = setLoop();
		}, staleTime || defaultStaleTime);
	}

	$effect(() => {
		timeoutId = setLoop();

		return () => {
			if (timeoutId != undefined) clearTimeout(timeoutId);
		};
	});

	if (target) {
		let isVisible = $state(false);

		observeIntersection(
			target,
			([entry]) => {
				isVisible = entry?.isIntersecting || false;
				if (isVisible) {
					if (timeoutId == undefined) timeoutId = setLoop();
				} else {
					if (timeoutId != undefined) clearTimeout(timeoutId);
					timeoutId = undefined;
				}
			},
			{}
		);
	}

	return {
		data,
		revalidate,
		isLoading
	};
}

export interface RecursiveQueryResult<D, IS> {
	data: Writable<D | undefined>;
	revalidate: () => Promise<D>;
	isLoading: Readable<boolean>;
	nextParam: Readable<IS | undefined>;
}

export interface RecursiveQueryOption<D, IS, S = IS> {
	initialParam: () => IS;
	nextParam: (lastPage: D) => IS;
	genParam: (param: IS, page: D) => S;
	fetcher: (param: IS | S, token?: string) => Promise<D>;
	key?: string[];
	staleTime?: number;
	/**
	 * fetch next page/refetch only when target element is visible
	 * @returns target element
	 */
	target?: () => HTMLElement | null | undefined;
	revalidateOnFocus?: boolean;
}

export function CreateRecursiveQuery<D, IS, S = IS>(
	option: RecursiveQueryOption<D, IS, S>
): RecursiveQueryResult<D, IS> {
	const {
		initialParam,
		fetcher,
		staleTime,
		target,
		genParam,
		nextParam: genNextParam,
		revalidateOnFocus,
		key
	} = option;

	let currentParam: S | undefined;
	const queryResult = CreateQuery({
		param: () => (currentParam == undefined ? initialParam() : currentParam),
		fetcher: async (param, token) => {
			let result = await fetcher(param, token);
			if (currentParam == undefined) currentParam = genParam(param as IS, result);
			return result;
		},
		staleTime,
		target,
		revalidateOnFocus,
		key
	});

	let canFetchNext = writable(target == undefined);

	if (target) {
		observeIntersection(
			target,
			([entry]) => {
				const isVisible = entry?.isIntersecting || false;
				canFetchNext.update((x) => x || isVisible);
			},
			{}
		);
	}

	let savedNextParam: Writable<IS | undefined> = writable(undefined);

	const nextParam = derived(
		[queryResult.data, canFetchNext, savedNextParam],
		([$page, $canFetchNext, $savedNextParam]: [D | undefined, boolean, IS | undefined]) => {
			if ($savedNextParam) return $savedNextParam;
			if ($canFetchNext && $page != undefined) {
				const nextParam = genNextParam($page);
				savedNextParam.set(nextParam);
				return nextParam;
			}
		}
	);

	return { nextParam, ...queryResult };
}

export interface InfiniteQueryOption<D, IS, S = IS> {
	initialParam: () => IS;
	nextParam: (lastPage: D) => IS;
	genParam: (param: IS, page: D) => S;
	fetcher: (param: IS | S, token?: string) => Promise<D>;
	key?: string[];
	staleTime?: number;
	revalidateOnFocus?: boolean;
}

export function CreateInfiniteQuery<D, IS, S = IS>(
	option: InfiniteQueryOption<D, IS, S>
): Readable<Array<RecursiveQueryResult<D, IS>>> {
	const head = CreateRecursiveQuery(option);
	const initialChain: RecursiveQueryResult<D, IS>[] = [head];

	const chainStore = readable<RecursiveQueryResult<D, IS>[]>(initialChain, (set) => {
		const subscriptions: (() => void)[] = [];

		const buildSubscription = (current: RecursiveQueryResult<D, IS>[]) => {
			const nextData = derived(current[current.length - 1].nextParam, (param) =>
				param == undefined
					? undefined
					: CreateRecursiveQuery({ ...option, initialParam: () => param })
			);
			const unsubscribe = nextData.subscribe((next) => {
				if (next) {
					const newChain = [...current, next];
					set(newChain);
					buildSubscription(newChain);
				}
			});

			subscriptions.push(unsubscribe);
			return unsubscribe;
		};

		const unsubscribeHead = buildSubscription(initialChain);
		subscriptions.push(unsubscribeHead);

		return () => {
			subscriptions.forEach((unsub) => unsub());
		};
	});

	return chainStore;
}
