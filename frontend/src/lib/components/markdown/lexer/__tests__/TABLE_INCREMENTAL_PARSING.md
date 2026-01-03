# Table Incremental Parsing - Bug Fixes and Expected Behavior

## Bug Fixed: Incremental Parsing Not Re-parsing Multi-line Blocks

### The Problem

The markdown parser supports incremental parsing for streaming content. However, when building tables line-by-line, the parser had a critical bug:

**Example of the bug:**
```typescript
// First parse: "| Header |" → Parsed as Paragraph
let result1 = await parseIncremental('| Header |', null);
// result1.tokens = [ParagraphToken]

// Second parse: "| Header |\n|---|---|" → Should be Table, but got [Paragraph, ?]
let result2 = await parseIncremental('| Header |\n|---|---|', result1.state);
// result2.tokens = [ParagraphToken, ...] // WRONG!
// Should be: result2.tokens = [TableToken]
```

### Root Cause

The incremental parser worked by:
1. Finding the "last complete region" in the previous parse
2. Keeping all tokens before that region unchanged
3. Only parsing new content from that point forward

This approach **failed for multi-line block tokens** where adding new lines changes the interpretation of previous lines:

- `| Header |` alone → Paragraph
- `| Header |\n|---|---|` → Table (the header line should be re-interpreted)

The parser didn't recognize that new content (table separator) should trigger re-parsing of the previous line.

## The Fix

### 1. Added `shouldReparseFromEarlier()` Function

This function detects when new content might change the interpretation of previous content:

```typescript
function shouldReparseFromEarlier(
	prevResult: ParseResult,
	newSource: string,
	oldSource: string
): boolean {
	// Check if last token is a paragraph ending at boundary
	if (lastToken.type === 'Paragraph' && lastToken.end >= oldSource.length - 2) {
		const lastLine = oldSource.slice(lastLineStart).trim();
		const hasPipes = lastLine.includes('|');
		
		if (hasPipes) {
			// Check if new content starts with table separator
			if (firstNewLine && /^[\|\s]*:?-+:?/.test(firstNewLine)) {
				return true; // Need to re-parse!
			}
		}
	}
	// ... more checks
}
```

### 2. Fixed Table Parser Minimum Row Requirement

**Bug:** Tables required at least 2 rows (header + 1 data row)
**Fix:** Tables now only require 1 row (header), matching GitHub Flavored Markdown

```typescript
// Before:
if (rows.length < 2) {  // ❌ Rejected header-only tables
	return null;
}

// After:
if (rows.length < 1) {  // ✅ Accepts header-only tables
	return null;
}
```

## Expected Behavior: Last Line as Paragraph First

As specified in the requirements:

> The expected behavior of a multiple line token (block-level token) is to parse last line as paragraph first, then re-parse as corresponding token if the line turns out to be part of block-level token

This behavior is now correctly implemented:

### Example 1: Table Formation
```typescript
Step 1: "| Header |"           → [Paragraph]
Step 2: "| Header |\n|---|---|" → [Table]  // Re-parsed from beginning
```

### Example 2: Incremental Row Addition
```typescript
Step 1: "| A | B |\n|---|---|\n| 1"     → [Table (incomplete)]
Step 2: "| A | B |\n|---|---|\n| 1 | 2 |" → [Table (complete)]
```

### Example 3: Mixed Content
```typescript
Step 1: "Text\n\n| Header"              → [Paragraph, Paragraph]
Step 2: "Text\n\n| Header |\n|---"     → [Paragraph, Paragraph (or incomplete)]
Step 3: "Text\n\n| Header |\n|---|---|\n| Data |" → [Paragraph, Table]
```

## Test Coverage

Added **29 comprehensive tests** covering:

### 1. The "Perfect Ratio" Table (from requirements)
- Complex table with 9 rows, 4 columns
- Bold and italic formatting in cells
- Special characters ($, K, commas)
- Alignment markers (`:---`)

### 2. Randomized Segmentation
- 10 random split points per test
- Verifies incremental parsing produces same result as full parse
- Tests various boundary conditions

### 3. Incremental Building Scenarios
- Row-by-row table construction
- Cell-by-cell completion
- Inline formatting added gradually

### 4. Content Mixing
- Tables with headings, paragraphs, lists, code blocks
- Tables within blockquotes
- Multiple tables in sequence

### 5. Format Variations
- Pipe-separated tables (`| A | B |`)
- Tab-separated tables (`A\tB`)
- Mixed tabs and spaces
- Alignment markers

### 6. Scale Testing
- Large tables (100+ rows)
- Wide tables (20+ columns)
- Incremental addition in chunks

### 7. Edge Cases
- Empty cells
- Special characters
- Incomplete tables
- Tables followed by other tables

## Test Results

All **286 tests** pass:
- ✅ 29 new table incremental parsing tests
- ✅ 257 existing tests (no regressions)

## Breaking Changes

**None.** The fixes are backwards compatible:
- Existing valid tables continue to parse correctly
- New feature: Header-only tables (previously invalid, now valid)
- Improved: Incremental parsing now handles more edge cases correctly
