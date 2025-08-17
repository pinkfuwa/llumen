import { type Readable, type Writable, derived, get } from 'svelte/store';
import { globalCache } from './cache';
import { onDestroy } from 'svelte';
import { isElementInViewport, once } from './helper';

export interface InternalQueryResult<T> {
	data: Writable<T | undefined>;
	revalidate: () => Promise<void>;
	isLoading: Readable<boolean>;
}

export interface InternalQueryOption<D> {
	fetcher: () => Promise<D | undefined>;
	key: string[];
	staleTime?: number;
	target?: Readable<HTMLElement | null>;
	revalidateOnFocus?: boolean | 'force';
	onSuccess?: (data?: D) => void;
	initialData?: D;
	cleanupCallback?: (cleanup: () => void) => void;
}

export function CreateInternalQuery<D>(option: InternalQueryOption<D>): InternalQueryResult<D> {
	let {
		target,
		key,
		staleTime,
		revalidateOnFocus,
		fetcher,
		onSuccess,
		initialData,
		cleanupCallback
	} = option;

	const cleanup = cleanupCallback ? cleanupCallback : onDestroy;

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
				const element = get(target);

				if (element == null || !isElementInViewport(element)) return Promise.resolve(undefined);
				return revalidate();
			}
		: revalidate;

	if (target) {
		const callback = once(
			target,
			(x) => x != null,
			(d) => {
				if (isElementInViewport(d!)) {
					revalidate();
					return;
				}
				const observer = new IntersectionObserver((entries) => {
					let isIntersecting = false;
					entries.forEach((entry) => {
						if (entry.isIntersecting) isIntersecting = true;
					});
					if (isIntersecting) {
						revalidate();
						observer.disconnect();
					}
				});
				observer.observe(d!);
			}
		);
		cleanup(callback);
	} else {
		revalidate();
	}

	if (staleTime != undefined) {
		let intervalId = setInterval(revalidateIfVisible, staleTime);

		cleanup(() => clearInterval(intervalId));
	}

	if (revalidateOnFocus) {
		const revalidateFnFocus = revalidateOnFocus == 'force' ? revalidate : revalidateIfVisible;

		window.addEventListener('focus', revalidateFnFocus);
		cleanup(() => window.removeEventListener('focus', revalidateFnFocus));
	}

	return {
		data,
		revalidate,
		isLoading
	};
}
