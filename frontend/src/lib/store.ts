import { writable, type Writable } from 'svelte/store';

export function localState<T>(
	key: string,
	defaultValue: T,
	checker: (data: T) => boolean = () => true
): Writable<T> {
	let storedValue = defaultValue;
	try {
		const storedRawVal = localStorage.getItem(key);
		if (storedRawVal != null) {
			const parsedVal = JSON.parse(storedRawVal);
			if (checker(parsedVal)) storedValue = parsedVal;
		}
	} catch (e) {
		console.warn(`localstorgae["${key}"] is invalid`);
	}

	const tokenStore = writable(storedValue);

	tokenStore.subscribe((value) => {
		if (value) {
			localStorage.setItem(key, JSON.stringify(value));
		} else {
			localStorage.removeItem(key);
		}
	});

	addEventListener('storage', (event) => {
		if (event.key === key) {
			tokenStore.set(JSON.parse(event.newValue!));
		}
	});

	return tokenStore;
}

export const token = localState<undefined | { value: string; expireAt: string; renewAt: string }>(
	'token',
	undefined
);