import { untrack } from 'svelte';
import { APIFetch } from './errorHandle';
import { isElementInViewport, addVisiblityListener, clearVisibilityListener } from './helper';

export interface QueryEffectOption<P, D> {
	path: string | (() => string);
	body?: P | (() => P);
	method?: 'POST' | 'GET' | 'PUT' | 'UPDATE';
	staleTime?: number;
	target?: () => HTMLElement | null;
	revalidateOnFocus?: boolean | 'force';
	onSuccess?: (data: D | undefined) => void;
	updateData: (data: D | undefined) => void;
}

/**
 * Creates a query effect that fetches data and updates state.
 * Must be called during component initialization.
 */
export function createQueryEffect<P, D>(
	option: QueryEffectOption<P, D>
): {
	revalidate: () => Promise<void>;
	isLoading: () => boolean;
} {
	let {
		target,
		staleTime = 60000,
		revalidateOnFocus = true,
		path,
		body,
		method,
		onSuccess,
		updateData
	} = option;

	const getPath = () => (path instanceof Function ? path() : path);
	const getBody = () => (body instanceof Function ? body() : body);

	let isLoading = $state(true);
	let hasInitialized = false;

	const fetcher = async () => {
		const data = await APIFetch<D>(getPath(), getBody(), method);
		if (onSuccess) onSuccess(data);
		updateData(data);
		isLoading = false;
		hasInitialized = true;
		return data;
	};

	const revalidate = async () => {
		await fetcher();
	};

	const revalidateIfVisible = target
		? () => {
				const element = untrack(target);
				if (element == null || !isElementInViewport(element)) return Promise.resolve(undefined);
				return revalidate();
			}
		: revalidate;

	// Initial fetch or intersection observer setup
	$effect(() => {
		const targetElement = target ? target() : null;

		if (target && targetElement) {
			if (isElementInViewport(targetElement)) {
				if (!hasInitialized) revalidate();
				return;
			}
			const observer = new IntersectionObserver((entries) => {
				let isIntersecting = false;
				entries.forEach((entry) => {
					if (entry.isIntersecting) isIntersecting = true;
				});
				if (isIntersecting) {
					if (!hasInitialized) revalidate();
					observer.disconnect();
				}
			});
			observer.observe(targetElement);
			return () => observer.disconnect();
		} else {
			if (!hasInitialized) revalidate();
		}
	});

	// Periodic revalidation
	$effect(() => {
		if (staleTime === undefined || staleTime === Infinity) return;

		let intervalId: ReturnType<typeof setInterval> | undefined = setInterval(
			revalidateIfVisible,
			staleTime
		);

		if (revalidateOnFocus) {
			const visibilityStateHandler = () => {
				if (document.visibilityState === 'visible' && intervalId === undefined) {
					intervalId = setInterval(revalidateIfVisible, staleTime);
				} else if (document.visibilityState === 'hidden' && intervalId !== undefined) {
					clearInterval(intervalId);
					intervalId = undefined;
				}
			};

			const visibilityListenerId = addVisiblityListener(visibilityStateHandler);
			return () => {
				if (intervalId !== undefined) clearInterval(intervalId);
				clearVisibilityListener(visibilityListenerId);
			};
		}

		return () => {
			if (intervalId !== undefined) clearInterval(intervalId);
		};
	});

	// Revalidate on focus
	$effect(() => {
		if (!revalidateOnFocus) return;

		const revalidateFnFocus = () => {
			if (document.visibilityState === 'hidden') return;
			if (revalidateOnFocus === 'force') revalidate();
			else revalidateIfVisible();
		};

		const visibilityListenerId = addVisiblityListener(revalidateFnFocus);
		return () => {
			clearVisibilityListener(visibilityListenerId);
		};
	});

	return {
		revalidate,
		isLoading: () => isLoading
	};
}
