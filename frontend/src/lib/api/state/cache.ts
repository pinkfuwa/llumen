import { writable, type Writable } from 'svelte/store';

const cache_size = 10;

type Entry = {
	store: Writable<any>;
	pending?: Promise<any> | undefined;
};

export class WritableCache {
	private map = new Map<string, Entry>();

	constructor(private maxSize = cache_size) {}

	// serialize key array deterministically
	private serialize(key: string[]) {
		// JSON is safe for arbitrary strings and keeps array structure
		return JSON.stringify(key);
	}

	// internal: get or create entry and mark it as most-recently-used
	private getEntry(key: string[]): Entry {
		const k = this.serialize(key);
		let entry = this.map.get(k);
		if (!entry) {
			entry = { store: writable(undefined) };
			this.map.set(k, entry);
			this.ensureSizeLimit();
		} else {
			// move to end to mark as recently used (Map preserves insertion order)
			this.map.delete(k);
			this.map.set(k, entry);
		}
		return entry;
	}

	// evict least-recently-used entries until within maxSize
	private ensureSizeLimit() {
		while (this.map.size > this.maxSize) {
			const oldestKey = this.map.keys().next().value as string | undefined;
			if (!oldestKey) break;
			const oldest = this.map.get(oldestKey);
			this.map.delete(oldestKey);
			// clear store value to help consumers notice eviction / free references
			try {
				oldest?.store.set(undefined);
			} catch {
				// ignore if store is already dead for some reason
			}
		}
	}

	// Public API

	get<T>(key: string[]): Writable<T | undefined> {
		const entry = this.getEntry(key);
		return entry.store as Writable<T | undefined>;
	}

	// overloads
	getOrExecute<T>(key: string[], f: () => Promise<T>): Writable<T | undefined>;
	getOrExecute<T>(key: string[], f: () => T): Writable<T>;
	getOrExecute<T>(key: string[], f: (() => Promise<T>) | (() => T)): Writable<T | undefined> {
		const entry = this.getEntry(key);

		// If there is already a pending computation, return the store (coalesce)
		if (entry.pending) {
			return entry.store as Writable<T | undefined>;
		}

		let result: Promise<T> | T;
		try {
			result = (f as any)();
		} catch (err) {
			// synchronous throw: do not set pending, rethrow
			throw err;
		}

		const isPromise = result && typeof (result as any).then === 'function';

		if (isPromise) {
			// set pending so concurrent callers share the same promise
			const p = (result as Promise<T>)
				.then((value) => {
					entry.store.set(value);
					entry.pending = undefined;
					return value;
				})
				.catch((err) => {
					entry.pending = undefined;
					// do not override existing value on error; rethrow so callers see the error
					throw err;
				});
			entry.pending = p;
			return entry.store as Writable<T | undefined>;
		} else {
			// synchronous result: set store immediately
			entry.store.set(result as T);
			return entry.store as Writable<T>;
		}
	}
}

export const globalCache = new WritableCache();
