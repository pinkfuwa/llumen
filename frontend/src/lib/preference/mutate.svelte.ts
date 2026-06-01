import { untrack } from 'svelte';
import { get, type Writable } from 'svelte/store';

export function propToRune<T, K extends keyof T>(store: Writable<T>, key: K): { val: T[K] };
export function propToRune<T, K1 extends keyof T, K2 extends keyof T[K1]>(
	store: Writable<T>,
	key1: K1,
	key2: K2
): { val: T[K1][K2] };
export function propToRune<T, K1 extends keyof T, K2 extends keyof T[K1]>(
	store: Writable<T>,
	key1: K1,
	key2?: K2
) {
	function propToRune1<T, K extends keyof T>(store: Writable<T>, key: K): { val: T[K] } {
		let rune = $state({ val: get(store)[key] });
		$effect(() => {
			store.update((x) => {
				x[key] = rune.val;
				return x;
			});
		});
		store.subscribe((value) => {
			if (value[key] !== untrack(() => rune.val)) rune.val = value[key];
		});

		return rune;
	}

	function propToRune2<T, K1 extends keyof T, K2 extends keyof T[K1]>(
		store: Writable<T>,
		key1: K1,
		key2: K2
	): { val: T[K1][K2] } {
		let rune = $state({ val: get(store)[key1][key2] });
		$effect(() =>
			store.update((x) => {
				x[key1][key2] = rune.val;
				return x;
			})
		);
		store.subscribe((value) => {
			if (value[key1][key2] !== untrack(() => rune.val)) rune.val = value[key1][key2];
		});

		return rune;
	}

	if (key2) return propToRune2(store, key1, key2);
	else return propToRune1(store, key1);
}
