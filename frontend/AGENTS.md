Frontend is built with Svelte 5, TailwindCSS, Vite, and TypeScript.

> You should read BUILD.md and `./development/svelte.md`.

## Svelte 5 Coding Guidelines

- Prioritize declarative code and reactivity clarity using Svelte 5 runes.
- Do not write comments that merely summarize code. Use them only to explain non-obvious "why" decisions, such as workarounds for edge cases.
- Embrace runes for state management: Use `$state` for mutable state, `$derived` for computed values, and `$effect` for side effects. Avoid Svelte 4-style reactivity patterns like `{@const}` or legacy bindings.
- Use full descriptive names for variables and props (e.g., `userProfile` instead of `up`). Follow camelCase for JS variables and kebab-case for component names/props.
- Always use **typescript**, and Never use `any` type
- `$effect` should be called during component initialization, `$state` should be at top-level(function...)
- Runes should be used in svelte or svelte.ts file
- Prefer using arrow function for callback(onclick/onpaste/...)
- To export `$derived`, use classes: https://svelte.dev/docs/svelte/$state#Classes
- To export `$state`, wrap state in `{val?: T}`

## Terminology

- Sync: the local has some changes, update those changes to server
- Fetch: get data from server
- Rune: svelte 5's reactivity pattern
- Store: svelte 4's reactivity pattern

## Commands to Run After Changes

- `pnpm check` - Run svelte-check for TypeScript validation
- `pnpm lint` - Check code formatting with prettier
- `pnpm format` - Format code with prettier
