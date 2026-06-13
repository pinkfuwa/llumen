## API Mutation Pattern

### Token Handling: Queries vs Mutations

`APIFetch`'s `token` option behaves differently depending on context (see `http.svelte.ts`):

- **`token: true`** — pulls token reactively from the `token` rune. Only valid inside `$effect.tracking()` (reactive context). Used in `$effect`, `$derived`, and `.then()` callbacks within those scopes.
- **`token: token.value?.value`** (a string) — passes the token value explicitly, breaking reactivity. Required in event handlers and other non-reactive contexts (mutation callbacks). When capturing token (or any signal) for mutations, wrap with `untrack()` from `svelte` to avoid establishing unwanted reactive dependencies.

### Rule of Thumb

- **Queries** (read data, inside `$effect.root` / `$derived`): use `token: true` + `.then()` for reactivity, so the request re-runs when token changes.
- **Mutations** (write data, from event handlers): capture `token.value?.value` inside `untrack()` first, pass the string. Use `.then()` IIFE instead of `async/await`:

```ts
// ✅ Mutation pattern
export function createUser(req: UserCreateReq): Promise<MutationStatus> {
	const token_ = untrack(() => token.value?.value);
	return APIFetch<UserCreateResp, UserCreateReq>({
		path: 'user/create',
		body: req,
		token: token_
	}).then((res) => {
		if (!res) return 'failed';
		// ... side effects
		return 'success';
	});
}
```

```ts
// ❌ Wrong: async/await + token: true outside effect tracking
export async function createUser(req: UserCreateReq): Promise<MutationStatus> {
	const res = await APIFetch<UserCreateResp, UserCreateReq>({
		path: 'user/create',
		body: req,
		token: true  // throws in dev mode outside $effect.tracking()
	});
}
```

### Per-Chat vs Per-Message Streaming

- **Per-chat `streaming`** — boolean from `message.svelte.ts`. When true, blocks all submissions across the chat (used in `Textbox.svelte` and `User.svelte` `disabled`).
- **Per-message `streaming`** — `msg.stream` boolean on individual messages. Enables slower incremental parsing for that message's rendering.

### Inline Short Functions

If a function is only used once at a single call site and is ≤3 lines, inline it directly:

```svelte
<!-- ✅ Inline -->
<CheckDelete ondelete={() => deleteUser({ user_id: user.id })} />

<!-- ❌ Unnecessary wrapper -->
async function handleDelete(userId: number) {
	await deleteUser({ user_id: userId });
}
<CheckDelete ondelete={() => handleDelete(user.id)} />
```

### Avoid Passing Props from `*.svelte.ts`

If a component needs data that lives in a `*.svelte.ts` module (rune state), the component should import it directly rather than receiving it as a prop. Props should only be for component-specific configuration, not for data already globally available.