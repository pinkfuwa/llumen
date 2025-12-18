# Citation Parsing Implementation

## Overview

The markdown parser now supports two formats for citations:

1. **Inline format**: `[@cite:id]` - Simple inline citation references
2. **Block format**: XML-like citation blocks with metadata

## Block Citation Format

Block citations use XML-like tags to provide rich metadata:

```markdown
<citation>
    <title>Citation Title</title>
    <url>https://example.com</url>
    <favicon>https://example.com/favicon.ico</favicon>
    <authoritative/>
</citation>
```

### Supported Fields

- **`<title>`** (optional): The citation title/name
- **`<url>`** (optional): The citation URL
- **`<favicon>`** (optional): URL to favicon for the citation source
- **`<authoritative/>`** (optional): Self-closing tag indicating authoritative source
  - Defaults to `false` if not present
  - Supports three formats: `<authoritative/>`, `<authoritative />`, `<authoritative></authoritative>`

### Example

```markdown
<citation>
    <title>澳門美食攻略：蛋撻、豬扒包、葡國料理推薦</title>
    <url>https://www.macaotourism.gov.mo/zh-hant/dining/feature-macau-cuisine</url>
    <favicon>https://www.macaotourism.gov.mo/favicon.ico</favicon>
    <authoritative/>
</citation>
```

## Implementation Details

### Token Structure

The `CitationToken` interface has been extended:

```typescript
export interface CitationToken extends Token {
    type: TokenType.Citation;
    id: string;           // Required: fallback identifier
    title?: string;       // Optional: citation title
    url?: string;         // Optional: citation URL
    favicon?: string;     // Optional: favicon URL
    authoritative?: boolean; // Optional: authoritative flag (defaults to false)
}
```

### Parser Implementation

1. **Block-level parsing** (`tryParseCitationBlock`):
   - Parses `<citation>...</citation>` blocks
   - Extracts nested tags using regex matching
   - Handles multiline content and whitespace
   - Returns `null` if closing tag is missing

2. **Inline parsing** (`tryParseCitation`):
   - Parses `[@cite:id]` format
   - Maintains backward compatibility
   - Sets optional fields to `undefined` and `authoritative` to `false`

### Rendering

The `Citation.svelte` component receives all fields from the token:

```svelte
<Citation
    raw={citToken.title || `[@${citToken.id}]`}
    title={citToken.title || `Citation ${citToken.id}`}
    url={citToken.url || ''}
    favicon={citToken.favicon || ''}
    authoritative={citToken.authoritative || false}
/>
```

The component displays:
- Authoritative citations with `SearchCheck` icon
- Non-authoritative citations with `Search` icon
- Favicon (if provided and loaded successfully)
- Title text

## Test Coverage

Comprehensive test suite in `frontend/src/lib/components/markdown/lexer/__tests__/citation.test.ts`:

- **22 citation-specific tests** covering:
  - Inline citation format
  - Block citation format with all field combinations
  - Multiple consecutive citations
  - Mixed content (citations with other markdown elements)
  - Edge cases (empty citations, missing tags, whitespace handling)
  - User-provided examples with Chinese characters and encoded URLs

### Test Organization

Tests have been reorganized into focused files:
- `citation.test.ts` - Citation parsing (22 tests)
- `blocks.test.ts` - Block-level tokens (32 tests)
- `inline.test.ts` - Inline tokens (39 tests)

Total: **244 tests** across all lexer test files, all passing.

## Files Changed

1. `frontend/src/lib/components/markdown/lexer/tokens.ts`
   - Extended `CitationToken` interface with optional fields

2. `frontend/src/lib/components/markdown/lexer/parser.ts`
   - Added `tryParseCitationBlock()` method
   - Updated `tryParseCitation()` to set default values
   - Integrated citation block parsing in `parseBlock()`

3. `frontend/src/lib/components/markdown/Parser.svelte`
   - Updated citation rendering to use token fields

4. `frontend/src/lib/components/markdown/lexer/__tests__/citation.test.ts`
   - New comprehensive test suite for citations

5. `frontend/src/lib/components/markdown/lexer/__tests__/blocks.test.ts`
   - New test suite for block-level tokens

6. `frontend/src/lib/components/markdown/lexer/__tests__/inline.test.ts`
   - New test suite for inline tokens

7. `frontend/src/lib/components/markdown/lexer/__tests__/README.md`
   - Documentation for test organization

## Backward Compatibility

The inline citation format `[@cite:id]` continues to work as before:
- Optional fields are set to `undefined`
- `authoritative` defaults to `false`
- ID is used for both `id` and fallback display

## Usage in Agent Context

This feature is designed for the deep research agent to provide rich citation metadata:

1. Agent generates citations with full metadata during research
2. Citations render with visual indicators (authoritative vs. non-authoritative)
3. Users can click citations to visit source URLs
4. Favicons provide visual recognition of sources
5. Authoritative sources are clearly distinguished