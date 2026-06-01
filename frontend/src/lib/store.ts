import { get, writable, type Writable } from 'svelte/store';

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

	const store = writable(storedValue);

	store.subscribe((value) => {
		if (value !== undefined) {
			localStorage.setItem(key, JSON.stringify(value));
		} else {
			localStorage.removeItem(key);
		}
	});

	addEventListener('storage', (event) => {
		if (event.key === key) {
			store.set(JSON.parse(event.newValue!));
		}
	});

	return store;
}

export const token = localState<undefined | { value: string; expireAt: string; renewAt: string }>(
	'token',
	undefined
);

export function syncState<T>(
	key: string,
	defaultValue: T,
	syncer: {
		upload: (data: T) => Promise<void>;
		download: () => Promise<T>;
	},
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

	let remoteValue: T | null = null;

	syncer.download().then((value) => {
		if (!checker(value)) return;
		remoteValue = value;
		store.set(value);
	});

	const store = writable(storedValue);

	store.subscribe((value) => {
		if (value !== undefined) {
			localStorage.setItem(key, JSON.stringify(value));
			if (JSON.stringify(value) !== remoteValue) syncer.upload(value);
		} else localStorage.removeItem(key);
	});

	addEventListener('storage', (event) => {
		if (event.key === key) {
			store.set(JSON.parse(event.newValue!));
		}
	});

	return store;
}

export function propToRune<T, K extends keyof T>(store: Writable<T>, key: K): T[K];
export function propToRune<T, K1 extends keyof T, K2 extends keyof T[K1]>(
	store: Writable<T>,
	key1: K1,
	key2: K2
): T[K1][K2];
export function propToRune<T, K1 extends keyof T, K2 extends keyof T[K1]>(
	store: Writable<T>,
	key1: K1,
	key2?: K2
) {
	function propToRune1<T, K extends keyof T>(store: Writable<T>, key: K): T[K] {
		let rune = $state(get(store)[key]);
		$effect(() =>
			store.update((x) => {
				x[key] = rune;
				return x;
			})
		);
		store.subscribe((value) => {
			if (value[key] !== rune) rune = value[key];
		});

		return rune;
	}

	function propToRune2<T, K1 extends keyof T, K2 extends keyof T[K1]>(
		store: Writable<T>,
		key1: K1,
		key2: K2
	): T[K1][K2] {
		let rune = $state(get(store)[key1][key2]);
		$effect(() =>
			store.update((x) => {
				x[key1][key2] = rune;
				return x;
			})
		);
		store.subscribe((value) => {
			if (value[key1][key2] !== rune) rune = value[key1][key2];
		});

		return rune;
	}

	if (key2) return propToRune2(store, key1, key2);
	else return propToRune1(store, key1);
}
