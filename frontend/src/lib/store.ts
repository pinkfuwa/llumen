import { localState } from '@sv-use/core';
import { writable } from 'svelte/store';

interface Current<T> {
	current: T;
}

export const useToken = () => {
	const storedValue = localStorage.getItem('token') || '';
	const tokenStore = writable(storedValue);

	tokenStore.subscribe((value) => {
		if (value) {
			localStorage.setItem('token', value);
		} else {
			localStorage.removeItem('token');
		}
	});

	return tokenStore;
};

export const useLanguage = () => localState('language', 'en') as Current<'en' | 'zh-tw'>;
export const useTheme = () => localState('theme', 'light') as Current<'light' | 'dark'>;
