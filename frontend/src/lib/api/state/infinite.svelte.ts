import { isElementInViewport } from './helper';

export interface Fetcher<D extends { id: number }> {
	range(startId: number, endId: number): Promise<D[] | undefined>;
	forward(limit: number, id?: number): Promise<D[] | undefined>;
	backward(limit: number, id: number): Promise<D[] | undefined>;
}

export interface PageState<D> {
	no: number;
	startId?: number;
	endId?: number;
	data: D[];
	target: HTMLElement | null;
}

export interface InfiniteQueryEffectOption<D extends { id: number }> {
	fetcher: Fetcher<D>;
	updatePages: (updater: (pages: PageState<D>[]) => PageState<D>[]) => void;
	getPages: () => PageState<D>[];
	initialId?: number | null;
	staleTime?: number;
	revalidateOnFocus?: boolean | 'force';
	maxPageSize?: number;
}

/**
 * Creates an infinite query effect that manages paginated data.
 * Must be called during component initialization.
 *
 * Page types:
 * - Initial page (no=0): both startId and endId undefined
 * - Forward page (no>=0): startId set, endId undefined
 * - Backward page (no<0): startId undefined, endId set
 * - Range page: both startId and endId set
 */
export function createInfiniteQueryEffect<D extends { id: number }>(
	option: InfiniteQueryEffectOption<D>
): void {
	const {
		fetcher,
		updatePages,
		getPages,
		initialId,
		staleTime,
		revalidateOnFocus = true,
		maxPageSize = 32
	} = option;

	// Initialize pages if empty
	// If initialId is undefined, treat it as null (create initial page with both undefined)
	// This allows fetching from the beginning and bidirectional scrolling
	if (getPages().length === 0) {
		const startId = initialId === undefined || initialId === null ? undefined : initialId;
		updatePages(() => [
			{
				no: 0,
				data: [],
				startId,
				endId: undefined,
				target: null
			}
		]);
	}

	// Track active page observers and fetch state
	const pageObservers = new Map<
		number,
		{ observer?: IntersectionObserver; intervalId?: number; isFetching: boolean }
	>();

	async function fetchAndUpdatePage(pageNo: number) {
		const pages = getPages();
		const page = pages.find((p) => p.no === pageNo);
		if (!page) return;

		const state = pageObservers.get(pageNo);
		if (state?.isFetching) return; // Prevent duplicate fetches
		if (state) state.isFetching = true;

		try {
			let newData: D[] | undefined;

			// Determine which fetcher to use based on page state
			if (page.endId !== undefined && page.startId !== undefined) {
				// Range page
				newData = await fetcher.range(page.startId, page.endId);
			} else if (page.endId !== undefined) {
				// Backward page
				newData = await fetcher.backward(maxPageSize, page.endId);
			} else {
				// Forward or initial page
				newData = await fetcher.forward(maxPageSize, page.startId);
			}

			if (newData === undefined || newData.length === 0) return;

			// Update page data and potentially add new pages
			updatePages((currentPages) => {
				const pageIndex = currentPages.findIndex((p) => p.no === pageNo);
				if (pageIndex === -1) return currentPages;

				let newPages = [...currentPages];
				let updatedPage = { ...currentPages[pageIndex], data: newData! };

				const pagesToPrepend: PageState<D>[] = [];
				const pagesToAppend: PageState<D>[] = [];

				// Check 1: Initial page special case - always add a backward page
				if (
					updatedPage.startId === undefined &&
					updatedPage.endId === undefined &&
					updatedPage.data.length > 0
				) {
					updatedPage.startId = newData![0].id;
					pagesToPrepend.push({
						no: updatedPage.no - 1,
						endId: newData![0].id + 1, // Assuming descending IDs
						startId: undefined,
						data: [],
						target: null
					});
				}

				// Check 2: Forward page with full data - add next forward page
				if (updatedPage.endId === undefined && newData!.length >= maxPageSize) {
					updatedPage.endId = newData!.at(-1)!.id;
					pagesToAppend.push({
						no: updatedPage.no + 1,
						startId: updatedPage.endId - 1, // Assuming descending IDs
						endId: undefined,
						data: [],
						target: null
					});
				}

				// Check 3: Backward page with full data - add previous backward page
				if (updatedPage.startId === undefined && newData!.length >= maxPageSize) {
					updatedPage.startId = newData![0].id;
					pagesToPrepend.push({
						no: updatedPage.no - 1,
						endId: updatedPage.startId + 1, // Assuming descending IDs
						startId: undefined,
						data: [],
						target: null
					});
				}

				// Update the page in the array
				newPages[pageIndex] = updatedPage;

				// Add new pages
				return [...pagesToPrepend, ...newPages, ...pagesToAppend];
			});
		} finally {
			const state = pageObservers.get(pageNo);
			if (state) state.isFetching = false;
		}
	}

	function activatePage(pageNo: number) {
		if (pageObservers.has(pageNo)) return;

		const pages = getPages();
		const page = pages.find((p) => p.no === pageNo);
		if (!page) return;

		// Initialize state for this page
		pageObservers.set(pageNo, { isFetching: false });

		// Setup intersection observer for lazy loading if target exists
		if (page.target) {
			const observer = new IntersectionObserver((entries) => {
				entries.forEach((entry) => {
					if (entry.isIntersecting) {
						fetchAndUpdatePage(pageNo);
					}
				});
			});

			observer.observe(page.target);

			const state = pageObservers.get(pageNo)!;
			state.observer = observer;

			// Setup periodic revalidation if configured
			if (staleTime !== undefined && staleTime !== Infinity) {
				const intervalId = window.setInterval(() => {
					const currentPage = getPages().find((p) => p.no === pageNo);
					if (
						currentPage?.target &&
						isElementInViewport(currentPage.target) &&
						document.visibilityState === 'visible'
					) {
						fetchAndUpdatePage(pageNo);
					}
				}, staleTime);

				state.intervalId = intervalId;
			}
		} else {
			// No target, fetch immediately
			fetchAndUpdatePage(pageNo);
		}
	}

	// Effect to activate pages when they're added
	$effect(() => {
		const pages = getPages();

		// Activate new pages
		pages.forEach((page) => {
			if (!pageObservers.has(page.no)) {
				activatePage(page.no);
			}
		});

		// Cleanup removed pages
		const currentPageNos = new Set(pages.map((p) => p.no));
		for (const [pageNo, state] of pageObservers) {
			if (!currentPageNos.has(pageNo)) {
				if (state.observer) state.observer.disconnect();
				if (state.intervalId !== undefined) clearInterval(state.intervalId);
				pageObservers.delete(pageNo);
			}
		}
	});

	// Cleanup on unmount
	$effect(() => {
		return () => {
			for (const state of pageObservers.values()) {
				if (state.observer) state.observer.disconnect();
				if (state.intervalId !== undefined) clearInterval(state.intervalId);
			}
			pageObservers.clear();
		};
	});

	// Revalidate on focus if configured
	$effect(() => {
		if (!revalidateOnFocus) return;

		const handleVisibilityChange = () => {
			if (document.visibilityState === 'visible') {
				const pages = getPages();
				pages.forEach((page) => {
					if (page.target && isElementInViewport(page.target)) {
						if (revalidateOnFocus === 'force') {
							fetchAndUpdatePage(page.no);
						}
					}
				});
			}
		};

		document.addEventListener('visibilitychange', handleVisibilityChange);
		return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
	});

	// Note: We do NOT need an effect to update observers when page targets change.
	// The target HTMLElement reference remains stable after initialization.
	// While the page object reference changes (due to immutable updates), the target
	// itself doesn't change. Creating such an effect would cause an infinite loop:
	// pages change -> effect runs -> observers recreated -> fetch triggered -> pages change -> loop
	// Observers are created once in activatePage() and that's sufficient.
}

/**
 * Insert data at the beginning of the first page
 */
export function insertInfiniteQueryData<D extends { id: number }>(
	pages: PageState<D>[],
	data: D
): PageState<D>[] {
	if (pages.length === 0) return pages;

	// Check if data already exists
	for (const page of pages) {
		if (page.data.some((d) => d.id === data.id)) return pages;
	}

	const newPages = [...pages];
	const firstPage = { ...newPages[0], data: [data, ...newPages[0].data] };
	newPages[0] = firstPage;

	return newPages;
}

/**
 * Update data by ID across all pages
 */
export function updateInfiniteQueryDataById<D extends { id: number }>(
	pages: PageState<D>[],
	id: number,
	updater: (data: D) => D
): PageState<D>[] {
	return pages.map((page) => {
		if (page.data.some((d) => d.id === id)) {
			return {
				...page,
				data: page.data.map((d) => (d.id === id ? updater(d) : d))
			};
		}
		return page;
	});
}

/**
 * Remove continuous data/pages from beginning where predicate returns true.
 * Removes entire pages that only contain matching data, filters the last page
 * that has some matching data, and includes all subsequent pages.
 */
export function removeInfiniteQueryData<D extends { id: number }>(
	pages: PageState<D>[],
	predicate: (data: D) => boolean
): PageState<D>[] {
	// Find the last page that contains any matching data
	let lastMatchingPageIndex = -1;
	for (let i = 0; i < pages.length; i++) {
		if (pages[i].data.some(predicate)) {
			lastMatchingPageIndex = i;
		} else {
			// Stop at first page with no matching data
			break;
		}
	}

	// If no pages have matching data, return unchanged
	if (lastMatchingPageIndex === -1) {
		return pages;
	}

	// Filter the last matching page to remove matching data
	const filteredData = pages[lastMatchingPageIndex].data.filter((d) => !predicate(d));

	// If filtered page has no data left, start from next page and reset startId
	if (filteredData.length === 0) {
		const remainingPages = pages.slice(lastMatchingPageIndex + 1);
		if (remainingPages.length > 0) {
			remainingPages[0] = {
				...remainingPages[0],
				startId: undefined
			};
		}
		return remainingPages;
	}

	// Return filtered page plus all subsequent pages
	const result = pages.slice(lastMatchingPageIndex);
	result[0] = {
		...result[0],
		data: filteredData,
		startId: undefined
	};
	return result;
}

/**
 * Get all data from pages in order
 */
export function getInfiniteQueryData<D extends { id: number }>(pages: PageState<D>[]): D[] {
	return pages.flatMap((page) => page.data);
}
