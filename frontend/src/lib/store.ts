import { writable, type Writable } from 'svelte/store';

function localState<T>(
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
export const locale = localState<'en' | 'zh-tw'>(
	'language',
	navigator.language.includes('zh') ? 'zh-tw' : 'en',
	(x) => x === 'en' || x === 'zh-tw'
);
export const theme = localState<'light' | 'dark' | 'orange' | 'blue'>(
	'theme',
	window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
	(x) => ['light', 'dark', 'orange', 'blue'].includes(x)
);
export const enterSubmit = localState<'true' | 'false'>('enterSubmit', 'false');
