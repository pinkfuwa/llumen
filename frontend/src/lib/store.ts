import { writable, type Writable } from 'svelte/store';
import { setTheme } from '$lib/theme';
import { setLocale } from '$lib/i18n';

function localState<T>(
	key: string,
	defaultValue: T,
	checker: (data: T) => boolean = () => true,
	onChange: (data: T) => void = () => {}
): () => Writable<T> {
	return () => {
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
			onChange(value);
		});

		addEventListener('storage', (event) => {
			if (event.key === key) {
				tokenStore.set(JSON.parse(event.newValue!));
			}
		});

		return tokenStore;
	};
}

export const useToken = localState<
	undefined | { value: string; expireAt: number; duration: number }
>('token', undefined);
export const useLanguage = localState<'en' | 'zh-tw'>(
	'language',
	navigator.language.includes('zh') ? 'zh-tw' : 'en',
	(x) => x === 'en' || x === 'zh-tw',
	setLocale
);
export const useTheme = localState<'light' | 'dark'>(
	'theme',
	'light',
	(x) => x === 'light' || x === 'dark',
	setTheme
);
