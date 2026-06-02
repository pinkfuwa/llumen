import { untrack } from 'svelte';

export type Val<T> = { val: T };

export type LocalSyncer<T> = {
	upload: (data: T) => Promise<void>;
	download: () => Promise<T>;
};

export interface LocalStateOption<T> {
	defaultValue(): T;
	checker?: (data: T) => boolean;
	syncer?: LocalSyncer<T>;
}

export type LocalStateHandle<T> = {
	value: T;
	sync(): Promise<void>;
};

function loadFromStorage<T>(key: string, defaultValue: () => T, checker: (data: T) => boolean): T {
	try {
		const raw = localStorage.getItem(key);
		if (raw !== null) {
			const parsed = JSON.parse(raw);
			if (checker(parsed)) return parsed as T;
		}
	} catch {
		console.warn(`localStorage["${key}"] is invalid`);
	}
	return defaultValue();
}

function saveToStorage<T>(key: string, value: T): void {
	try {
		localStorage.setItem(key, JSON.stringify(value));
	} catch (e) {
		console.warn(`localStorage["${key}"] save failed`, e);
	}
}

export function localState<T>(key: string, option: LocalStateOption<T>): LocalStateHandle<T> {
	const checker = option.checker ?? (() => true);
	const initial = loadFromStorage(key, option.defaultValue, checker);
	const handle: LocalStateHandle<T> = $state({
		value: initial,
		sync: async () => {}
	});
	let lastSerialized = JSON.stringify($state.snapshot(handle.value));
	let syncing = false;

	$effect.root(() => {
		$effect(() => {
			const snap = $state.snapshot(handle.value);
			const serialized = JSON.stringify(snap);
			if (serialized === lastSerialized) return;
			lastSerialized = serialized;
			saveToStorage(key, snap);
			if (option.syncer && !syncing) {
				option.syncer.upload(snap as T);
			}
		});
		return () => {};
	});

	if (typeof window !== 'undefined') {
		window.addEventListener('storage', (event) => {
			if (event.key === key && event.newValue != null) {
				try {
					const parsed = JSON.parse(event.newValue);
					if (checker(parsed)) {
						untrack(() => {
							handle.value = parsed;
							lastSerialized = event.newValue!;
						});
					}
				} catch {}
			}
		});
	}

	handle.sync = async () => {
		if (!option.syncer) return;
		const remote = await option.syncer.download();
		if (!checker(remote)) return;
		syncing = true;
		untrack(() => {
			handle.value = remote;
			lastSerialized = JSON.stringify($state.snapshot(handle.value));
			saveToStorage(key, remote);
		});
		queueMicrotask(() => {
			syncing = false;
		});
	};

	return handle;
}

export type TokenInfo = { value: string; expireAt: string; renewAt: string };

function loadToken(): TokenInfo | undefined {
	return loadFromStorage<TokenInfo | undefined>(
		'token',
		() => undefined,
		() => true
	);
}

export const token: LocalStateHandle<TokenInfo | undefined> = $state({
	value: loadToken(),
	sync: async () => {}
});

let tokenLastSerialized = JSON.stringify($state.snapshot(token.value));

$effect.root(() => {
	$effect(() => {
		const snap = $state.snapshot(token.value);
		const serialized = JSON.stringify(snap);
		if (serialized === tokenLastSerialized) return;
		tokenLastSerialized = serialized;
		saveToStorage('token', snap);
	});
	return () => {};
});

if (typeof window !== 'undefined') {
	window.addEventListener('storage', (event) => {
		if (event.key === 'token' && event.newValue != null) {
			try {
				token.value = JSON.parse(event.newValue);
				tokenLastSerialized = event.newValue;
			} catch {}
		}
	});
}
