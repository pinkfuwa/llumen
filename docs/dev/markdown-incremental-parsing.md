# Markdown Incremental Parsing

## Overview

The markdown component supports two parsing modes:

1. **Non-incremental (default)**: Full parse in a web worker
2. **Incremental**: Fake incremental parsing in the main thread with throttling

## Architecture

### Code Structure

- `frontend/src/lib/components/markdown/Root.svelte`: State management and rendering
- `frontend/src/lib/components/markdown/lexer/index.ts`: Pure parsing functions (async for code splitting)
- `frontend/src/lib/components/markdown/worker/`: Web worker implementation for non-incremental parsing

### Design Principles

1. **Clear separation**: Lexer contains pure functions, Root.svelte manages state
2. **Code splitting**: All lexer functions are async to enable dynamic imports
3. **Non-blocking**: Incremental parsing runs in main thread with throttling; non-incremental in worker

## How Incremental Parsing Works

### Region Boundaries

The parser tracks "complete regions" for each block-level element:

- **Paragraphs**: Separated by blank lines
- **Code blocks**: Fenced with ``` markers
- **Tables**: Complete table structures
- **Lists**: Ordered and unordered lists
- **Blockquotes**: Quote blocks

These regions serve as natural splitting points for incremental parsing.

### Algorithm

```typescript
parseIncremental(source: string, state: IncrementalState | null) -> IncrementalParseResult
```

1. **Full reparse conditions**:
   - No previous state exists
   - New source doesn't extend previous source (text was deleted/modified)
   - New content is only whitespace

2. **Incremental path**:
   - Find the last complete region in previous parse result
   - Parse only content after that region
   - Adjust token positions by offsetting with region end position
   - Combine stable tokens with newly parsed tokens

3. **State maintenance**:
   - Store previous source, parse result, and content position
   - Update state after each parse for next incremental operation

### Example

```typescript
// First parse
const source1 = "# Title\n\nParagraph one.";
const result1 = await parseIncremental(source1, null);

// Incremental parse (extends previous)
const source2 = source1 + "\n\nParagraph two.";
const result2 = await parseIncremental(source2, result1.state);
// Only parses "Paragraph two." and reuses first two tokens
```

## Usage

### In Components

```svelte
<script>
import Root from '$lib/components/markdown/Root.svelte';

let content = $state("# Hello\n\nWorld");
</script>

<!-- Non-incremental (uses web worker) -->
<Root source={content} />

<!-- Incremental (main thread, throttled) -->
<Root source={content} incremental={true} />
```

### Throttling

Incremental mode uses 100ms throttling to batch rapid updates (e.g., during typing):

- Updates are queued
- Parse executes after 100ms of inactivity
- If new content arrives during parsing, schedules another parse

## Performance Characteristics

### Non-incremental Mode

- **Pros**: Non-blocking (runs in worker), full featured
- **Cons**: Message passing overhead, can't share state
- **Best for**: Initial renders, large documents, non-streaming content

### Incremental Mode

- **Pros**: Reuses parsing work, faster for small additions
- **Cons**: Runs in main thread (mitigated by throttling), "fake" incremental
- **Best for**: Live typing, streaming responses, gradual content updates

### "Fake" Incremental

Called "fake" because it doesn't do true incremental parsing (character-by-character). Instead:

1. Detects complete regions (paragraphs, tables, etc.)
2. Reuses tokens for complete regions
3. Reparses from last complete boundary

This provides good performance for streaming use cases without the complexity of true incremental parsing.

## Region Boundary Detection

The `findLastCompleteRegion()` function uses a multi-step approach:

1. **Region-based**: Prefers using tracked regions (tables, code blocks, etc.)
2. **Token-based**: Falls back to finding well-separated tokens
3. **Conservative**: Returns 0 if no clear boundary exists (triggers full reparse)

A region is "complete" if:
- It ends before the current source end (not the last characters)
- It has proper closing syntax (e.g., blank line after paragraph)

## Testing

Comprehensive tests in `lexer/incremental.test.ts` cover:

- State initialization and updates
- Region detection (code blocks, tables, lists, blockquotes, paragraphs)
- Token position adjustment
- Multiple sequential incremental parses
- Edge cases (empty input, whitespace-only updates)

Run tests:
```bash
cd frontend
pnpm test lexer/incremental.test.ts
```

## Future Improvements

Potential enhancements:

1. **True incremental parsing**: Character-level incremental updates
2. **Adaptive throttling**: Adjust delay based on document size
3. **Partial render**: Render only changed regions
4. **Worker-based incremental**: Move incremental logic to worker
5. **Better heuristics**: Smarter detection of complete regions

## Related Files

- `frontend/src/lib/components/markdown/Root.svelte`: Main component
- `frontend/src/lib/components/markdown/lexer/index.ts`: Parsing logic
- `frontend/src/lib/components/markdown/lexer/parser.ts`: Parser implementation
- `frontend/src/lib/components/markdown/lexer/tokens.ts`: Token and region types
- `frontend/src/lib/components/markdown/worker/`: Web worker implementation