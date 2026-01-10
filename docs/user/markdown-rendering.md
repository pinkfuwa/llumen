# Markdown Rendering

## Overview

Llumen includes a custom markdown parser and renderer optimized for LLM chat applications. It supports standard markdown with additional features for mathematical notation, citations, and streaming content.

## Features

### Standard Markdown

All standard markdown features are supported:

#### Headings
```markdown
# Heading 1
## Heading 2
### Heading 3
```

#### Text Formatting
```markdown
**Bold text** or __bold text__
*Italic text* or _italic text_
~~Strikethrough text~~
`inline code`
```

#### Lists
```markdown
- Unordered list item
- Another item

1. Ordered list item
2. Second item
```

#### Links and Images
```markdown
[Link text](https://example.com)
![Alt text](image.png)
```

#### Blockquotes
```markdown
> This is a quote
> It can span multiple lines
```

#### Code Blocks
````markdown
```javascript
function hello() {
  console.log("Hello, world!");
}
```
````

#### Tables
```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
```

**Note**: Llumen also supports tab-separated tables (non-standard):
```markdown
Header 1	Header 2
--------	--------
Cell 1		Cell 2
```

#### Horizontal Rules
```markdown
---
***
___
```

### LaTeX Math

Mathematical notation using LaTeX syntax:

#### Inline Math
```markdown
The equation \( E = mc^2 \) is famous.
Or: The value $x = 5$ is constant.
```

**Note**: Llumen uses relaxed spacing rules for inline math with `$` delimiters, allowing expressions like `$x=5$` without spaces.

#### Block Math
```markdown
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

Or using `\[` and `\]`:
```markdown
\[
\sum_{i=1}^n i = \frac{n(n+1)}{2}
\]
```

### Citations (Custom)

Reference citations in text:

```markdown
This is a claim [@cite:123].
Multiple citations [@cite:abc] are [@cite:xyz] supported.
```

Citations are rendered with special styling and can be used for references or footnotes.

## Rendering Modes

### Normal Mode (Default)

```svelte
<Root source={markdownText} />
```

- Uses web worker for parsing (non-blocking)
- Best for static or infrequently updated content
- Full reparse on every update

### Incremental Mode

```svelte
<Root source={markdownText} incremental={true} />
```

- Parses in main thread with throttling
- Optimized for streaming/live updates
- Reuses previous parsing work when content extends
- 100ms update throttle to batch rapid changes

**When to use incremental mode:**
- Live typing in editor
- Streaming LLM responses
- Gradual content updates
- Real-time collaborative editing

**When to use normal mode:**
- Static content display
- Large documents loaded at once
- Content that changes completely (not just appends)

### Code Block Highlighting

Code blocks are syntax highlighted automatically when they are **closed** (have both opening and closing ``` delimiters). Unclosed code blocks remain unhighlighted, which is particularly useful during streaming:

- **Closed blocks**: Immediately highlighted with full syntax support
- **Unclosed blocks**: Displayed in monochrome until closing delimiter appears
- This provides instant visual feedback when a code block is complete during streaming responses

## Performance Tips

### For Large Documents

- Use normal mode (web worker) to keep UI responsive
- Consider lazy loading or pagination for very large documents

### For Streaming Content

- Enable incremental mode for better performance
- Content updates are automatically throttled
- Parser reuses work from previous parses

### For Real-time Editing

- Incremental mode provides smooth typing experience
- 100ms throttle reduces unnecessary parses
- Region-based parsing minimizes recomputation

## Styling

Markdown components use Tailwind CSS classes and can be customized via:

1. **CSS overrides**: Target component classes in your styles
2. **Theme customization**: Modify Tailwind config for global changes

## Technical Details

### Parser Architecture

- **Lexer**: Tokenizes markdown into AST
- **Parser**: Builds token tree with position tracking
- **Renderer**: Svelte components for each token type

### Region Boundaries

The parser tracks "complete regions" for incremental parsing:
- Paragraphs (separated by blank lines)
- Code blocks (complete fenced blocks)
- Tables (header + separator + rows)
- Lists (consecutive list items)
- Blockquotes (quote blocks)

These regions allow the parser to reuse work when content extends.

### Position Tracking

All tokens include start/end positions for:
- Error reporting
- Syntax highlighting
- Interactive features (future)

## Limitations

### Known Edge Cases

1. **Tables**: Require at least one data row (header + separator alone is invalid)
2. **Nested lists**: Limited depth support
3. **LaTeX**: Complex multi-line expressions may need `\[...\]` delimiters
4. **Citations**: Must follow `[@cite:id]` format exactly

### Not Supported

- HTML tags (for security)
- Task lists (`- [ ]` checkboxes)
- Footnotes (use citations instead)
- Definition lists
- Automatic URL linking (use explicit `[text](url)`)

## Examples

### Chat Message with Mixed Content

```markdown
# Analysis Results

Here are the findings from the data:

1. Mean value: $\mu = 42.5$
2. Standard deviation: $\sigma = 3.2$

The distribution follows:

$$
f(x) = \frac{1}{\sigma\sqrt{2\pi}} e^{-\frac{(x-\mu)^2}{2\sigma^2}}
$$

According to the literature [@cite:smith2023], this is expected.

```python
import numpy as np
data = np.random.normal(42.5, 3.2, 1000)
print(f"Mean: {data.mean()}")
```

> **Note**: Results may vary based on sample size.
```

### Streaming Response

When displaying a streaming LLM response:

```svelte
<script>
let response = $state("");

// Simulate streaming
async function streamResponse() {
  const chunks = ["# ", "Hello", "\n\n", "This is ", "streaming."];
  for (const chunk of chunks) {
    response += chunk;
    await new Promise(r => setTimeout(r, 100));
  }
}
</script>

<Root source={response} incremental={true} />
```

The incremental mode efficiently handles the gradual content updates.

## Troubleshooting

### Slow Rendering

- Check document size (consider pagination)
- Try incremental mode for streaming content
- Ensure web worker is loading correctly (check console)

### Math Not Rendering

- Verify LaTeX syntax is correct
- Use `\[...\]` for complex block equations
- Check for unescaped special characters

### Tables Not Parsing

- Ensure at least one data row exists
- Check separator row has correct format: `|---|---|`
- Verify proper alignment of columns

### Incremental Mode Issues

- Incremental mode works best with appended content
- Deletions or edits trigger full reparse
- Very large documents may still be slow (use normal mode)

## Related Documentation

- [Architecture: Markdown Component](../dev/markdown-incremental-parsing.md)
- [Testing: Markdown Parser](../../frontend/src/lib/components/markdown/lexer/parser.test.ts)