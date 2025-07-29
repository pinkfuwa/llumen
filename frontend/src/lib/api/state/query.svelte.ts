import { type Readable, get, derived, readable, writable } from 'svelte/store';
import { useSWR } from 'sswr';
import { observeIntersection } from '@sv-use/core';
import { useToken } from '$lib/store';

const defaultStaleTime = 30000;

export interface QueryResult<T> {
	data: Readable<T | undefined>;
	revalidate: () => Promise<T>;
	isLoading: Readable<boolean>;
}

export interface useQueryOption<P, D> {
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

export function useQuery<P, D>(option: useQueryOption<P, D>): QueryResult<D> {
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

export interface InfiniteQueryResult<D, IS> {
	data: Readable<D | undefined>;
	revalidate: () => Promise<D>;
	isLoading: Readable<boolean>;
	nextParam: Readable<IS | undefined>;
}

export interface useInfiniteQueryOption<D, IS, S = IS> {
	initialParam: IS;
	nextParam: (lastPage: D) => IS;
	genParam: (param: IS, page: D) => S;
	fetcher: (param: IS | S, token?: string) => Promise<D>;
	key?: string[];
	staleTime?: number;
	/**
	 * refetch only when targetElementIsVisible
	 * @returns target element
	 */
	target?: () => HTMLElement | null | undefined;
	revalidateOnFocus?: boolean;
}

export function useInfiniteQuery<D, IS, S = IS>(
	option: useInfiniteQueryOption<D, IS, S>
): InfiniteQueryResult<D, IS> {
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
	const queryResult = useQuery({
		param: () => (currentParam == undefined ? initialParam : currentParam),
		fetcher: async (param, token) => {
			let result = await fetcher(param, token);
			if (currentParam == undefined) currentParam = genParam(initialParam, result);
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

	const nextParam = derived(
		[queryResult.data, canFetchNext],
		([$page, $canFetchNext]: [D | undefined, boolean]) =>
			!$canFetchNext || $page == undefined ? undefined : genNextParam($page)
	);

	return { nextParam, ...queryResult };
}
