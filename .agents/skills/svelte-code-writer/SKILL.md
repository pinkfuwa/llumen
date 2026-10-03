---
name: svelte-code-writer
description: CLI tools for Svelte 5 documentation lookup and code analysis. MUST be used whenever creating, editing or analyzing any Svelte component (.svelte) or Svelte module (.svelte.ts/.svelte.js). If possible, this skill should be executed within the svelte-file-editor agent for optimal results.
---

## CLI tools

You have access to `@sveltejs/mcp` CLI for Svelte-specific assistance. Use these commands via `pnpm dlx`:

### List documentation sections

```bash
pnpm dlx @sveltejs/mcp list-sections
```

Lists all available Svelte 5 and SvelteKit documentation sections with titles and paths.

### Get documentation

```bash
pnpm dlx @sveltejs/mcp get-documentation '<section1>,<section2>,...'
```

Retrieves full documentation for specified sections. Use after `list-sections` to fetch relevant docs.

**Example:**

```bash
pnpm dlx @sveltejs/mcp get-documentation '$state,$derived,$effect'
```

### Svelte autofixer

```bash
pnpm dlx @sveltejs/mcp svelte-autofixer '<code_or_path>' [options]
```

Analyzes Svelte code and suggests fixes for common issues.

**Options:**

- `--async` - Enable async Svelte mode (default: false)
- `--svelte-version` - Target version: 4 or 5 (default: 5)

**Examples:**

```bash
# Analyze inline code (single quotes preserve $)
pnpm dlx @sveltejs/mcp svelte-autofixer '<script>let count = $state(0);</script>'

# Analyze a file
pnpm dlx @sveltejs/mcp svelte-autofixer ./src/lib/Component.svelte

# Target Svelte 4
pnpm dlx @sveltejs/mcp svelte-autofixer ./Component.svelte --svelte-version 4
```

**Important:** In Bash, single-quote arguments containing runes (`$state`, `$derived`, etc.) to prevent shell variable substitution. Do not add backslashes before `$` inside single quotes. For code containing single quotes, pass the file path instead.

## Workflow

1. **Uncertain about syntax?** Run `list-sections` then `get-documentation` for relevant topics
2. **Reviewing/debugging?** Run `svelte-autofixer` on the code to detect issues
3. **Always validate** - Run `svelte-autofixer` before finalizing any Svelte component
