import { get, writable, type Writable } from 'svelte/store';

const cache_size = 100;

type Entry = {
	store: Writable<any>;
};

export class WritableCache {
	private map = new Map<string, Entry>();

	constructor(private maxSize = cache_size) {}

	// serialize key array deterministically
	private serialize(key: string[]) {
		// JSON is safe for arbitrary strings and keeps array structure
		return JSON.stringify(key);
	}

	private hasEntry(key: string[]): boolean {
		const k = this.serialize(key);
		return this.map.has(k);
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
			oldest?.store.set(undefined);
		}
	}

	// clear the entire cache and reset stores so consumers see the cleared state
	clear(): void {
		this.map.clear();
	}

	// Public API

	get<T>(key: string[]): Writable<T | undefined> {
		const entry = this.getEntry(key);
		return entry.store as Writable<T | undefined>;
	}

	getOr<T>(key: string[], val: T): Writable<T> {
		const hasKey = this.hasEntry(key);
		const entry = this.getEntry(key);

		if (!hasKey) entry.store.set(val);
		return entry.store as Writable<T>;
	}

	getOrExecute<T>(key: string[], f: () => T): Writable<T> {
		const hasKey = this.hasEntry(key);
		const entry = this.getEntry(key);

		if (!hasKey) entry.store.set(f());
		return entry.store as Writable<T>;
	}
}

export const globalCache = new WritableCache();
