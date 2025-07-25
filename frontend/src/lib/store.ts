import { writable, type Updater, type Writable } from 'svelte/store';
import { browser } from '$app/environment';

interface Current<T> extends Writable<T> {
	value: T;
}

function createStore(key: string, defaultValue: string): Current<string> {
	const { subscribe, set, update } = writable<string>(
		browser ? localStorage.getItem(key) || defaultValue : defaultValue
	);

	return {
		subscribe,
		set: (value: string) => {
			localStorage.setItem(key, value);
			set(value);
		},
		update: (updater: Updater<string>) => {
			let newValue = updater(localStorage.getItem(key) || defaultValue);
			localStorage.setItem(key, newValue);
			update(() => newValue);
		},
		get value() {
			let currentValue;
			subscribe((value) => {
				currentValue = value;
			})();
			return currentValue!;
		}
	};
}

export const token = createStore('token', '');

export const language = createStore('language', 'en') as Current<'en' | 'zh-tw'>;
export const theme = createStore('theme', 'light') as Current<'light' | 'dark'>;
