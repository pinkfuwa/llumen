frontend has a module `http.svelte.ts`, which is just a HTTP client.

Most importantly, http svelte expose a token option:
- if set to string, use provided string as token(mutation)
- if set to true, try to pull token reactively from rune(query state)
- if set to false(default), use no-token(login/pre-auth)

The reason behind the design is to explicitly make the caller choose to load signal, so it's reactive.
