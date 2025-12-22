# Prompt Caching Optimization

## Problem

Chat completions were not hitting cached prefixes because variable content (time, chat_title) was included in the main system prompt. This prevented effective prompt caching since the system prompt changed with every request.

## Solution

We implemented a context injection pattern that separates static system prompts from variable context information:

### 1. Created `context.j2` Template

A new template (`agent/prompt/context.j2`) contains all variable content:
- Current date/time (with improved precision)
- Chat title
- Llumen-related context (conditionally included)

### 2. Refactored Existing Prompts

Removed variable content from:
- `agent/prompt/normal.j2`
- `agent/prompt/search.j2`

These prompts are now static and cacheable by LLM providers.

### 3. Context Injection

The context is rendered separately and injected as a **user message** at the end of the message history, but before the actual user query. This happens only for:
- Normal mode
- Search mode

Deep research mode does not use this pattern since it already has different caching characteristics.

### 4. Improved Time Precision

Enhanced time format for better context awareness:

- **Normal/Search context**: `[weekday], [hour]:[minute], [day] [month] [year]`
  - Example: "Monday, 14:30, 15 January 2024"
  
- **Deep research**: `[weekday], [hour]:[minute]:[second], [day] [month] [year]`
  - Example: "Monday, 14:30:45, 15 January 2024"
  - Higher precision for detailed research workflows

- **System prompt (legacy)**: `[weekday], [year]-[month]-[day]`
  - Day-level precision for maximum caching (deprecated in normal/search)

### 5. Smart Llumen Context Detection

Introduced `llumen_related` variable that detects if the user query mentions Llumen:
- Checks for "llumen" (case-insensitive)
- Checks for "流明" (Traditional Chinese)
- Checks for "app" (case-insensitive)

When detected, comprehensive Llumen documentation is included in the context message.

## Implementation Details

### Code Changes

**`backend/src/chat/prompt.rs`:**
- Added `PromptKind::Context` enum variant
- Added `render_context()` method with smart llumen detection
- Updated time format constants for better precision
- Added `ContextRenderingContext` struct

**`backend/src/chat/configs/configuration.rs`:**
- Modified `process()` method to inject context as user message
- Context injection happens after converting DB messages
- Only applies to `PromptKind::Normal` and `PromptKind::Search`

**`agent/prompt/context.j2`:**
- New template containing variable content
- Conditional llumen context based on `llumen_related` flag
- Clean separation of concerns

### Message Flow (Normal/Search Mode)

```
[System Message]     ← Static, cacheable prompt (normal.j2 or search.j2)
[User Message 1]     ← From chat history
[Assistant Message 1]
[User Message 2]
[Assistant Message 2]
...
[Context Message]    ← Injected from context.j2 (variable content)
[Current User Query] ← Latest user input
```

## Benefits

1. **Improved Cache Hit Rate**: Static system prompts can be cached by LLM providers (OpenRouter, etc.)
2. **Reduced Costs**: Cached tokens are typically much cheaper than new tokens
3. **Faster Responses**: Cached prefixes reduce processing time
4. **Better Time Context**: More precise timestamps for relevant modes
5. **Smarter Context**: Only includes Llumen docs when relevant

## Testing

To verify cache hits:
1. Check OpenRouter dashboard for cache statistics
2. Monitor response times (should decrease with cache hits)
3. Verify cost reduction in usage metrics

## Future Improvements

- Replace hardcoded Llumen context with RAG system
- Add more sophisticated context detection patterns
- Consider caching strategy for deep research mode
- Implement per-user context preferences