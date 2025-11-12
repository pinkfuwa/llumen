llumen is a LLM chat application. featuring three mode: normal chat, search and deep research.

# Rust coding guidelines

* Prioritize code correctness and clarity. Speed and efficiency are secondary priorities unless otherwise specified.
* Do not write organizational or comments that summarize the code. Comments should only be written in order to explain "why" the code is written in some way in the case there is a reason that is tricky / non-obvious.
* Prefer implementing functionality in existing files unless it is a new logical component. Avoid creating many small files.
* Avoid using functions that panic like `unwrap()`, instead use mechanisms like `?` to propagate errors.
* Be careful with operations like indexing which may panic if the indexes are out of bounds.
* Never silently discard errors with `let _ =` on fallible operations. Always handle errors appropriately:
  - Propagate errors with `?` when the calling function should handle them
  - Use `.log_err()` or similar when you need to ignore errors but want visibility
  - Use explicit error handling with `match` or `if let Err(...)` when you need custom logic
  - For error cause by user's bad request body, send the rrror by return `Json<Error>`
  Example:
  ```rust
  pub trait WithKind<T> {
      fn kind(self, kind: ErrorKind) -> Result<T, Json<Error>>;
      fn raw_kind(self, kind: ErrorKind) -> Result<T, Error>;
  }
  impl<T, E> WithKind<T> for Result<T, E>;
  ```
  - For error cause by false output of LLM, place insert chunk to top-level message.
* Never create files with `mod.rs` paths - prefer `src/some_module.rs` instead of `src/some_module/mod.rs`.
* Use full words for variable names (no abbreviations like "q" for "queue")
* Always use `cargo add` to add new dependency, which should automatically chose latest version
* Use variable shadowing to scope clones in async contexts for clarity, minimizing the lifetime of borrowed references.
  Example:
  ```rust
  executor.spawn({
      let task_ran = task_ran.clone();
      async move {
          *task_ran.borrow_mut() = true;
      }
  });
  ```

# Svelte 5 coding guidelines

* Prioritize declarative code and reactivity clarity using Svelte 5 runes.
* Do not write comments that merely summarize code. Use them only to explain non-obvious "why" decisions, such as workarounds for edge cases.
* Prefer extending existing components unless introducing a new logical unit (e.g., a distinct UI module). Avoid proliferating small, single-purpose components; consolidate where logical.
* Embrace runes for state management: Use `$state` for mutable state, `$derived` for computed values, and `$effect` for side effects. Avoid Svelte 4-style reactivity patterns like `{@const}` or legacy bindings.
* Use full descriptive names for variables and props (e.g., `userProfile` instead of `up`). Follow camelCase for JS variables and kebab-case for component names/props.
* Always use **typescript**, and Never use `any` type
* For shared state across components, prefer lightweight stores (`writable` or `readable`) only when necessary (e.g., app-wide data). Otherwise, use props and runes for local reactivity to minimize global dependencies.
* Handle asynchronous operations declaratively with custom query management library.
* Never ignore errors; use effects or snippets, and expose user-friendly feedback via components.
  - Propagate critical errors (backend disconnect/reachable) with `dispatchError` from `frontend/src/lib/error.ts`.
  - For user-input errors (e.g., form validation), display inline messages without crashing the app.

## query management library

`frontend/src/lib/api/state/index.ts` emphasizes small bundle sizes by omitting advanced features like automatic deduplication in queries and signal-based inputs.
Instead, it relies on manual top-level invocation (e.g., via setContext) for sharing queries across components.

```ts
export function useRoom(id: number): QueryResult<ChatReadResp> {
	return CreateQuery<ChatReadReq, ChatReadResp>({
		key: ['chatRead', id.toString()],
		path: 'chat/read',
		body: { id },
		revalidateOnFocus: false,
		staleTime: Infinity
	});
}
```

## Database government

We use sea-orm without entity-codegen. When modifying migration files, ensure they are compatible with the current database schema. don't generate entity-codegen.

## Strong Size Awareness

Our project emphasizes performance and binary-size. We choose minimal dependency for both frontend and backend.

### backend tech stack
* Rust
* Axum
* SeaORM
* Tokio
* mLua with luau sandboxing

### frontend tech stack
* Svelte 5(SPA)
* TailwindCSS
* Vite
* TypeScript
* bit-ui
* lezer(for markdown parsing)

## Document your changes

* document user-facing changes in `docs/user.md`
* document architecture changes in `docs/design.md`
