import { type Readable, type Writable, derived, get } from 'svelte/store';
import { globalCache } from './cache';
import { onDestroy } from 'svelte';
import { isElementInViewport, once, addVisiblityListener, clearVisibilityListener } from './helper';

export interface InternalQueryResult<T> {
	data: Writable<T | undefined>;
	revalidate: () => Promise<void>;
	isLoading: Readable<boolean>;
}

export interface InternalQueryOption<D> {
	fetcher: () => Promise<D | undefined>;
	key: string[] | Writable<D>;
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

	const data = key instanceof Array ? globalCache.get<D>(key) : key;

	if (initialData && !(initialData instanceof Array)) data.set(initialData);
	if (onSuccess && get(data) != undefined) onSuccess(get(data)!);

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

	if (staleTime != undefined && staleTime != Infinity) {
		let intervalId: ReturnType<typeof setInterval> | undefined = setInterval(
			revalidateIfVisible,
			staleTime
		);

		if (revalidateOnFocus) {
			const visibilityStateHandler = () => {
				if (document.visibilityState == 'visible' && intervalId == undefined) {
					intervalId = setInterval(revalidateIfVisible, staleTime);
				} else if (document.visibilityState == 'hidden' && intervalId != undefined) {
					clearInterval(intervalId);
					intervalId = undefined;
				}
			};

			let visibilityListenerId: number | undefined = addVisiblityListener(visibilityStateHandler);
			cleanup(() => {
				if (visibilityListenerId != undefined) clearVisibilityListener(visibilityListenerId);
			});
		}

		cleanup(() => {
			if (intervalId != undefined) clearInterval(intervalId);
		});
	}

	if (revalidateOnFocus) {
		const revalidateFnFocus = () => {
			if (document.visibilityState == 'hidden') return;
			if (revalidateOnFocus == 'force') revalidate();
			else revalidateIfVisible();
		};

		let visibilityListenerId2: number | undefined = addVisiblityListener(revalidateFnFocus);
		cleanup(() => {
			if (visibilityListenerId2 != undefined) clearVisibilityListener(visibilityListenerId2);
		});
	}

	return {
		data,
		revalidate,
		isLoading
	};
}

export function SetQueryData<D>(option: {
	key: string[];
	updater: (data: D | undefined) => D | undefined;
}) {
	const { key, updater } = option;
	const data = globalCache.get<D>(key);
	data.update(updater);
}
