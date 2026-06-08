# Svelte 5 Patterns

## `$derived` in classes

Top-level `$derived` cannot be exported. Wrap in a class:

```ts
export class WidgetState {
	label = $derived.by(() => someCondition ? a : b);
	count = $derived(base + offset);
	item = $derived(list?.find((x) => x.id === this.count) ?? null);
}
export const widget = new WidgetState();
```

`$derived` can be reassigned (unless `const`), so `bind:` works on it.

Cross-referencing sibling derived fields requires `this.`:

```ts
export class WidgetState {
	a = $derived(computeA());
	b = $derived(this.a + 1); // ✅ this.a
}
```

Omitting `this` silently references a module-scope variable.

## `{#key}` — force destroy/recreate

Destroys and recreates children when the expression changes. Use when a component needs to reset internal state that isn't reactive:

```svelte
{#key pageId}
	<PageViewer />
{/key}
```

Note: `{#key}` causes re-initialization, not prevents it.

## `{#each}` — always provide a key

```svelte
{#each items as item (item.id)}
	<ItemView {item} />
{/each}
```

Without a key, Svelte uses index-based diffing — incorrect when items are inserted/removed/reordered.

Note: key does not prevent re-initialization. When a new array is assigned, all items re-init regardless. Key only helps match items across renders when the same array is mutated in place.
