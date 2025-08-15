import { type Readable, type Writable, derived, get } from 'svelte/store';
import { globalCache } from './cache';
import { onDestroy } from 'svelte';

function isElementInViewport(element: HTMLElement) {
	var rect = element.getBoundingClientRect();

	return (
		rect.top >= 0 &&
		rect.left >= 0 &&
		rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
		rect.right <= (window.innerWidth || document.documentElement.clientWidth)
	);
}

export interface InternalQueryResult<T> {
	data: Writable<T | undefined>;
	revalidate: () => Promise<void>;
	isLoading: Readable<boolean>;
}

export interface InternalQueryOption<D> {
	fetcher: () => Promise<D | undefined>;
	key: string[];
	staleTime?: number;
	target?: Readable<HTMLElement | null> | (() => HTMLElement | null);
	revalidateOnFocus?: boolean | 'force';
	onSuccess?: (data?: D) => void;
	initialData?: D;
}

export function CreateInternalQuery<D>(option: InternalQueryOption<D>): InternalQueryResult<D> {
	let { target, key, staleTime, revalidateOnFocus, fetcher, onSuccess, initialData } = option;

	const data = globalCache.get<D>(key);
	if (initialData) data.set(initialData);
	const isLoading = derived([data], ([data]) => data === undefined);

	const revalidate = async () => {
		const newData = await fetcher();
		if (onSuccess) onSuccess(newData);
		if (newData) data.set(newData);
	};

	const revalidateIfVisible = target
		? () => {
				const element = target instanceof Function ? target() : get(target);

				if (element == null || !isElementInViewport(element)) return Promise.resolve(undefined);
				return revalidate();
			}
		: revalidate;

	if (staleTime != undefined) {
		let intervalId = setInterval(revalidateIfVisible, staleTime);

		onDestroy(() => clearInterval(intervalId));
	}

	if (revalidateOnFocus) {
		const revalidateFnFocus = revalidateOnFocus == 'force' ? revalidate : revalidateIfVisible;

		window.addEventListener('focus', revalidateFnFocus);
		onDestroy(() => window.removeEventListener('focus', revalidateFnFocus));
	}

	return {
		data,
		revalidate,
		isLoading
	};
}
