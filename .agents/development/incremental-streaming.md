# Streaming Incremental Rendering

Design decisions and gotchas for streaming markdown with Svelte 5.

## Only the incremental codeblock flash is the bug

The flash/glitch when a code block re-initializes during **streaming** is the bug.  
The flash on **message complete** (final full re-parse) is **expected and acceptable**.

## How Svelte 5 `{#each}` compares items

```svelte
{#each nodes as node}
```

`{#each}` compares each item with `===` (reference identity).  
Adding a key (or using default index-keying) does **not** prevent re-initialization - if the array contains a new object at the same index, the component is destroyed and re-created. Avoid index keys; they are functionally identical to no key.

**Consequence**: To avoid component re-initialization during streaming, the same `AstNode` objects must persist across ticks. Mutate them in-place instead of replacing them.

## True incremental, not full re-parse + tree diff

`smd.ts` is a streaming markdown parser designed from the ground up for incremental feeding. The renderer (`renderer.ts`) should be changed accordingly - feed only the delta (new characters) each tick, keep the parser state machine alive, and let callbacks append/mutate nodes in-place.

Do **not** re-parse the entire source every tick and diff the trees (`patchAst`). That approach has bad performance and defeats the streaming parser's design.

## `closed` tracking is rejected

Do not add a `closed` flag to `StackEntry` or the renderer. The user explicitly rejected this.

## Block-level tokens close independently of message completion

"block level-token may close even if message is not complete, they are two different things."

The parser closes a code block (or other block-level token) when it encounters the closing delimiter (e.g. ` ``` `), **not** when the message stream finishes. These are distinct events:

- **Token close**: The parser saw a closing delimiter and called `end_token`. Handled by the normal parser state machine.
- **Message complete**: All characters have been received. Handled by `parser_end` or a full re-parse.

Do not conflate the two. Do not use `end_token` for message completion logic.

## `parser_end` for message completion, not `end_token`

When the message is complete, use `parser_end(p)` to flush pending state and finalize remaining open tokens. Or do a full re-parse (`parseSync`). Do not rely on `end_token` callbacks for this purpose.

## Code block `language` must be synced on the node

`set_attr(LANG, ...)` is called by the parser when it sees the language specifier after ` ``` `. The renderer's `set_attr` handler sets `entry.language` on the stack entry, but this must be propagated to the `AstNode` immediately (for streaming) and in `finalizeNodeInPlace` (for correct finalization).

The `CodeBlockNode`'s `content` should also be synced incrementally in `add_text` so components see partial content before `end_token`.

## No `.splice(0)` in `ensureParser`

When the streaming parser is first created (`ensureParser()`), do **not** clear `rootChildren` with `.splice(0)`. Doing so means `prevLength = 0` and the delta becomes the entire source - a full re-parse from scratch, which violates the incremental feed contract.

During normal streaming, `source` starts empty and grows incrementally, so `rootChildren` is already empty when the parser is created.

## Feed only delta during incremental; full re-parse only on explicit reparse

**During streaming**: Feed only `delta = source.slice(prevLength)` to the persistent parser. Never feed the full source.

**On reparse** (message complete, user edit, error recovery): Do a full `parseSync` - this replaces `rootChildren` entirely with fresh nodes.
