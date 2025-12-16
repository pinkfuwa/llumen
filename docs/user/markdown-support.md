# Markdown Support

Llumen provides rich markdown rendering support for chat messages, including standard markdown syntax and several extensions for academic and technical writing.

## Standard Markdown

### Headings

Use `#` symbols to create headings from level 1 to 6:

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

### Text Formatting

- **Bold**: `**bold text**` or `__bold text__`
- *Italic*: `*italic text*` or `_italic text_`
- ~~Strikethrough~~: `~~strikethrough text~~`
- `Inline code`: `` `code` ``

### Lists

**Unordered lists:**
```markdown
- Item 1
- Item 2
  - Nested item
- Item 3
```

You can also use `*` or `+` instead of `-`.

**Ordered lists:**
```markdown
1. First item
2. Second item
3. Third item
```

### Links and Images

**Links:**
```markdown
[Link text](https://example.com)
```

**Images:**
```markdown
![Alt text](image.jpg)
```

### Blockquotes

```markdown
> This is a blockquote
> It can span multiple lines
```

### Code Blocks

Use triple backticks for code blocks. Optionally specify a language for syntax highlighting:

````markdown
```javascript
function hello() {
  console.log("Hello, world!");
}
```
````

### Horizontal Rules

Create horizontal rules with three or more dashes, asterisks, or underscores:

```markdown
---
***
___
```

### Tables

Create tables using pipes `|` or tabs:

**Pipe-separated:**
```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

**Tab-separated:**
```markdown
Header 1	Header 2	Header 3
---	---	---
Cell 1	Cell 2	Cell 3
Cell 4	Cell 5	Cell 6
```

## LaTeX Math Support

Llumen has built-in support for LaTeX mathematical expressions, perfect for technical and scientific discussions.

### Inline Math

For inline mathematical expressions, you can use:

- `\( ... \)` - Standard LaTeX inline math
- `$ ... $` - Dollar notation (with relaxed spacing rules)

**Examples:**
```markdown
The equation \(E = mc^2\) is famous.
Einstein's formula is $E = mc^2$.
The quadratic formula is $x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}$.
```

### Block Math

For display equations, use:

- `\[ ... \]` - Standard LaTeX display math
- `$$ ... $$` - Dollar notation (must be on separate lines)

**Examples:**
```markdown
\[
E = mc^2
\]

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$
```

### Relaxed Spacing Rules

Unlike strict markdown parsers, Llumen uses relaxed spacing rules for convenience:

- ✅ `$x$` - Works without spaces
- ✅ `$x^2$` - Works for simple expressions
- ✅ `$x + y$` - Works with internal spaces
- ❌ `$ x$` - Space immediately after `$` is invalid
- ❌ `Price $ 5.00` - Space before `$` prevents it from being LaTeX

This makes it easy to write mathematical expressions naturally without worrying about spacing.

## Citations (Experimental)

For academic and research discussions, Llumen supports a custom citation syntax:

```markdown
According to the study [@smith2020], the results indicate...
Multiple sources [@smith2020; @jones2021] support this conclusion.
```

Citations can be rendered as interactive elements with metadata when connected to a citation database.

## Tips for Best Results

1. **Leave blank lines** - Separate different block elements (paragraphs, lists, code blocks) with blank lines for proper formatting.

2. **Preview as you type** - Llumen renders markdown in real-time, so you can see your formatting immediately.

3. **Mix and match** - You can combine markdown with LaTeX and other features freely:
   ```markdown
   # Physics Basics
   
   The energy-momentum relation is:
   
   $$
   E^2 = (pc)^2 + (m_0c^2)^2
   $$
   
   This is fundamental to **special relativity** [@einstein1905].
   ```

4. **Code highlighting** - Always specify the language for code blocks to get proper syntax highlighting:
   ````markdown
   ```python
   def fibonacci(n):
       if n <= 1:
           return n
       return fibonacci(n-1) + fibonacci(n-2)
   ```
   ````

5. **Tables for data** - Use tables to present structured data clearly. Both pipe and tab formats work equally well.

## Limitations

- Nested lists have limited depth support
- HTML tags are not supported for security reasons
- Some advanced GFM (GitHub Flavored Markdown) features like task lists are not yet supported
- Footnotes are not currently supported

## Getting Help

If you encounter any issues with markdown rendering or have suggestions for improvements, please report them through the feedback system or GitHub issues.