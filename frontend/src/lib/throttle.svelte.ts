import { untrack } from 'svelte';

export function useThrottle<T extends unknown[]>(fn: (...args: T) => void, ms: number) {
	let lastCall = 0;
	let timeout: ReturnType<typeof setTimeout> | undefined = undefined;
	let lastArgs: T = [] as unknown as T;

	function throttled(...args: T) {
		untrack(() => {
			const now = Date.now();
			const nextAllowed = lastCall + ms;

			if (now < nextAllowed) {
				lastArgs = args;
				if (!timeout) {
					timeout = setTimeout(() => {
						timeout = undefined;
						lastCall = Date.now();
						fn(...lastArgs);
					}, nextAllowed - now);
				}
				return;
			}

			if (timeout) {
				clearTimeout(timeout);
				timeout = undefined;
			}

			lastCall = now;
			fn(...args);
		});
	}

	throttled.cancel = () => {
		if (timeout) {
			clearTimeout(timeout);
			timeout = undefined;
		}
	};

	return throttled;
}
