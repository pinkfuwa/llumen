# LaTeX Parsing Rules

This document describes the parsing rules for LaTeX expressions in the markdown lexer.

## Overview

The lexer supports four types of LaTeX delimiters:
- **Block delimiters**: `$$...$$` and `\[...\]`
- **Inline delimiters**: `$...$` and `\(...\)`

## Block LaTeX Delimiters

### `$$` Delimiter

The `$$` delimiter creates a block-level LaTeX expression.

**Requirements:**
- Must have a newline immediately after the opening `$$`
- Must have a closing `$$`

**Valid examples:**
```markdown
$$
y^2 + x^2
$$
```

**Invalid examples:**
```markdown
$$ inline $$        # No newline after opening $$
$$ x^2 $$          # Space instead of newline after opening $$
```

### `\[` and `\]` Delimiter

The `\[...\]` delimiter creates a block-level LaTeX expression.

**Requirements:**
- Does NOT require a newline after the opening `\[`
- Must have a closing `\]`

**Valid examples:**
```markdown
\[
x^2 + y^2
\]

\[x^2\]            # Same line is also valid
```

## Inline LaTeX Delimiters

### `\(` and `\)` Delimiter

The `\(...\)` delimiter creates an inline LaTeX expression.

**Requirements:**
- Must have a closing `\)`
- No spacing requirements
- Can appear anywhere in text

**Valid examples:**
```markdown
Text \(x^2\) more
word\(x\)word
\(a + b\)
```

### `$` Delimiter

The `$...$` delimiter creates an inline LaTeX expression with specific spacing rules.

**Requirements:**

1. **Must not be `$$`** (that's a block delimiter)
2. **Must have closing `$`**
3. **Content cannot be empty**
4. **Symmetric spacing rule**: Content must either:
   - Start and end with space: `$ x + y $` ✓
   - Start and end without space: `$x+y$` ✓
   - Asymmetric spacing is rejected: `$ x$` ✗, `$x $` ✗

5. **External spacing rule** (when content has whitespace):
   - If content contains any whitespace, then:
     - If not at start of text: must have space before opening `$`
     - If not at end of text: must have space after closing `$`

**Valid examples:**
```markdown
$x$                           # No spaces inside or outside
$x^2$                         # No spaces inside
word$x^2$word                 # No spaces inside, works anywhere
The equation $E=mc^2$ is...   # No spaces inside
Text $ x + y $ more           # Spaces inside, spaces outside
$ x + y $ more                # At start, space after closing $
Text $ x + y $                # At end, space before opening $
```

**Invalid examples:**
```markdown
I have $1 and $               # Asymmetric: ends with space, no char after closing $
$ x$                          # Asymmetric: starts with space, doesn't end with space
$x $                          # Asymmetric: ends with space, doesn't start with space
word$ x + y $ more            # Has spaces inside but no space before opening $
Text $ x + y $word            # Has spaces inside but no space after closing $
Price $100                    # Unclosed
Text $$ more                  # Empty content (or $$ block delimiter)
$1 and $5                     # Ambiguous: spaces inside, no surrounding spaces
I have $5 and you have $10    # Multiple invalid: no spaces after each closing $
```

## Implementation Details

The inline `$` delimiter parsing logic (in `tryParseInlineLatex`):

1. Reject if followed by another `$` (that's `$$`)
2. Find the closing `$`
3. Reject if content is empty
4. Check for asymmetric spacing in content
5. If content has whitespace:
   - Check for space before opening `$` (unless at text start)
   - Check for space after closing `$` (unless at text end)

## Test Coverage

Comprehensive test coverage is provided in `latex.test.ts` with test groups:
- Block delimiter tests (7 tests)
- Inline delimiter tests (3 tests)
- `$` delimiter without spaces inside (5 tests)
- `$` delimiter with spaces inside (7 tests)
- `$` edge cases (6 tests)
- Multiple delimiters (3 tests)
- Content edge cases (3 tests)

Total: 34 dedicated LaTeX tests + existing parser tests