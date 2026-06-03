# Frontend AGENTS

Frontend is built with Svelte 5, TailwindCSS, Vite, and TypeScript.

## Svelte 5 Coding Guidelines

- Prioritize declarative code and reactivity clarity using Svelte 5 runes.
- Do not write comments that merely summarize code. Use them only to explain non-obvious "why" decisions, such as workarounds for edge cases.
- Prefer extending existing components unless introducing a new logical unit (e.g., a distinct UI module). Avoid proliferating small, single-purpose components; consolidate where logical.
- Embrace runes for state management: Use `$state` for mutable state, `$derived` for computed values, and `$effect` for side effects. Avoid Svelte 4-style reactivity patterns like `{@const}` or legacy bindings.
- Use full descriptive names for variables and props (e.g., `userProfile` instead of `up`). Follow camelCase for JS variables and kebab-case for component names/props.
- Always use **typescript**, and Never use `any` type
- For shared state across components, prefer lightweight stores (`writable` or `readable`) only when necessary (e.g., app-wide data). Otherwise, use props and runes for local reactivity to minimize global dependencies.
- Handle asynchronous operations declaratively with custom query management library.
- Never ignore errors; use effects or snippets, and expose user-friendly feedback via components.
  - Propagate critical errors (backend disconnect/reachable) with `dispatchError` from `frontend/src/lib/error.ts`.
  - For user-input errors (e.g., form validation), display inline messages without crashing the app.
- `$effect` should be called during component initialization, `$state` should be at top-level(function...)
- Runes should be used in svelte or svelte.ts file
- Prefer using arrow function for callback(onclick/onpaste/...)

## Commands to Run After Changes

- `pnpm check` - Run svelte-check for TypeScript validation
- `pnpm lint` - Check code formatting with prettier
- `pnpm format` - Format code with prettier
