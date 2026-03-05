import { writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

/** @type {{ name: string; markdown: string; expectedTypes: string[]; deepExpectedPaths?: string[][] }[]} */
const templates = [
	{
		name: 'heading-1',
		markdown: '# Heading 1',
		expectedTypes: ['Heading']
	},
	{
		name: 'heading-2',
		markdown: '## Heading 2',
		expectedTypes: ['Heading']
	},
	{
		name: 'heading-3',
		markdown: '### Heading 3',
		expectedTypes: ['Heading']
	},
	{
		name: 'heading-4',
		markdown: '#### Heading 4',
		expectedTypes: ['Heading']
	},
	{
		name: 'heading-5',
		markdown: '##### Heading 5',
		expectedTypes: ['Heading']
	},
	{
		name: 'heading-6',
		markdown: '###### Heading 6',
		expectedTypes: ['Heading']
	},
	{
		name: 'paragraph-simple',
		markdown: 'Just a paragraph.',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'paragraph-multiline',
		markdown: 'Line one\nLine two',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'paragraph-multiple',
		markdown: 'First paragraph\n\nSecond paragraph',
		expectedTypes: ['Paragraph', 'Paragraph']
	},
	{
		name: 'code-block-js',
		markdown: '```javascript\nconst x = 1;\n```',
		expectedTypes: ['CodeBlock']
	},
	{
		name: 'code-block-no-lang',
		markdown: '```\nplain code\n```',
		expectedTypes: ['CodeBlock']
	},
	{
		name: 'code-block-unclosed',
		markdown: '```python\ndef foo():\n    pass',
		expectedTypes: ['CodeBlock']
	},
	{
		name: 'hr-dashes',
		markdown: '---',
		expectedTypes: ['HorizontalRule']
	},
	{
		name: 'hr-stars',
		markdown: '***',
		expectedTypes: ['HorizontalRule']
	},
	{
		name: 'hr-underscores',
		markdown: '___',
		expectedTypes: ['HorizontalRule']
	},
	{
		name: 'blockquote-single',
		markdown: '> A quote',
		expectedTypes: ['Blockquote']
	},
	{
		name: 'blockquote-multi',
		markdown: '> Line 1\n> Line 2',
		expectedTypes: ['Blockquote']
	},
	{
		name: 'ul-basic',
		markdown: '- Alpha\n- Beta\n- Gamma',
		expectedTypes: ['UnorderedList']
	},
	{
		name: 'ul-star',
		markdown: '* One\n* Two',
		expectedTypes: ['UnorderedList']
	},
	{
		name: 'ol-basic',
		markdown: '1. First\n2. Second\n3. Third',
		expectedTypes: ['OrderedList']
	},
	{
		name: 'ol-start-5',
		markdown: '5. Five\n6. Six',
		expectedTypes: ['OrderedList']
	},
	{
		name: 'table-basic',
		markdown: '| A | B |\n|---|---|\n| 1 | 2 |',
		expectedTypes: ['Table']
	},
	{
		name: 'table-multi-row',
		markdown: '| X | Y | Z |\n|---|---|---|\n| a | b | c |\n| d | e | f |',
		expectedTypes: ['Table']
	},
	{
		name: 'latex-block-dollar',
		markdown: '$$\nE = mc^2\n$$',
		expectedTypes: ['LatexBlock']
	},
	{
		name: 'latex-block-bracket',
		markdown: '\\[\n\\sum_{i=0}^n i\n\\]',
		expectedTypes: ['LatexBlock']
	},
	{
		name: 'inline-bold',
		markdown: '**bold text**',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-italic',
		markdown: '*italic text*',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-strike',
		markdown: '~~struck~~',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-code',
		markdown: '`code`',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-link',
		markdown: '[click](https://example.com)',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-autolink',
		markdown: '<https://example.com>',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-image',
		markdown: '![alt](https://img.png)',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-latex-paren',
		markdown: 'The \\(x^2\\) formula',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-latex-dollar',
		markdown: 'Inline $x^2$ math',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'inline-br',
		markdown: 'line1<br>line2',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'mixed-heading-para-hr',
		markdown: '# Title\n\nBody text.\n\n---',
		expectedTypes: ['Heading', 'Paragraph', 'HorizontalRule']
	},
	{
		name: 'mixed-code-then-list',
		markdown: '```\ncode\n```\n\n- item',
		expectedTypes: ['CodeBlock', 'UnorderedList']
	},
	{
		name: 'mixed-quote-then-table',
		markdown: '> quoted\n\n| A | B |\n|---|---|\n| 1 | 2 |',
		expectedTypes: ['Blockquote', 'Table']
	},
	{
		name: 'latex-in-table',
		markdown: '| Expr | Val |\n|---|---|\n| \\(|x|\\) | 42 |',
		expectedTypes: ['Table']
	},
	{
		name: 'latex-block-dollar-no-newline',
		markdown: '$$x^2 + y^2 = z^2$$',
		expectedTypes: ['LatexBlock']
	},
	{
		name: 'latex-block-bracket-no-newline',
		markdown: '\\[E = mc^2\\]',
		expectedTypes: ['LatexBlock']
	},
	{
		name: 'latex-block-dollar-sentence-prefix',
		markdown: 'Here is the theorem:\n$$x^2$$',
		expectedTypes: ['Paragraph', 'LatexBlock']
	},
	{
		name: 'latex-block-bracket-sentence-prefix',
		markdown: 'Consider:\n\\[F = ma\\]',
		expectedTypes: ['Paragraph', 'LatexBlock']
	},
	{
		name: 'latex-inline-dollar-mid-sentence',
		markdown: 'The value is $$42$$ in total.',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'latex-dollar-not-price',
		markdown: 'This costs $5 and $10 total.',
		expectedTypes: ['Paragraph']
	},
	{
		name: 'table-cell-bold-inlinecode',
		markdown: '| **`code`** | plain |\n|---|---|\n| x | y |',
		expectedTypes: ['Table'],
		deepExpectedPaths: [['Table', 'TableRow', 'TableCell', 'Bold', 'InlineCode']]
	},
	{
		name: 'table-cell-bold-latexinline',
		markdown: '| **\\(x^2\\)** | value |\n|---|---|\n| 5 | 6 |',
		expectedTypes: ['Table'],
		deepExpectedPaths: [['Table', 'TableRow', 'TableCell', 'Bold', 'LatexInline']]
	},
	{
		name: 'table-header-cell-italic-latexinline',
		markdown: '| *\\(a\\)* | *\\(b\\)* |\n|---|---|\n| 1 | 2 |',
		expectedTypes: ['Table'],
		deepExpectedPaths: [['Table', 'TableRow', 'TableCell', 'Italic', 'LatexInline']]
	},
	{
		name: 'blockquote-para-bold-latexinline',
		markdown: '> **The formula \\(E=mc^2\\)** is famous.',
		expectedTypes: ['Blockquote'],
		deepExpectedPaths: [['Blockquote', 'Paragraph', 'Bold', 'LatexInline']]
	},
	{
		name: 'blockquote-table-cell-bold',
		markdown: '> | **strong** | normal |\n> |---|---|\n> | x | y |',
		expectedTypes: ['Blockquote'],
		deepExpectedPaths: [['Blockquote', 'Table', 'TableRow', 'TableCell', 'Bold']]
	},
	{
		name: 'blockquote-table-header-bold-latex',
		markdown: '> | **\\(x\\)** | \\(y\\) |\n> |---|---|\n> | a | b |',
		expectedTypes: ['Blockquote'],
		deepExpectedPaths: [['Blockquote', 'Table', 'TableRow', 'TableCell', 'Bold', 'LatexInline']]
	},
	{
		name: 'heading-bold-latexinline',
		markdown: '## The **\\(x^2\\)** theorem',
		expectedTypes: ['Heading'],
		deepExpectedPaths: [['Heading', 'Bold', 'LatexInline']]
	},
	{
		name: 'list-item-bold-inlinecode',
		markdown: '- **item with `code`**',
		expectedTypes: ['UnorderedList'],
		deepExpectedPaths: [['UnorderedList', 'ListItem', 'Bold', 'InlineCode']]
	},
	{
		name: 'sample-full-doc',
		markdown: `# Isotropy Subgroup, Orbit, Action, and Divisor – Quick Guide

> **In abstract algebra**, these notions formalise how a group "moves" elements of a set and how symmetry behaves locally.

---

## 1. Group Action

**Definition**  
A **group action** of a group \\(G\\) on a set \\(X\\) is a function
\\[
\\cdot : G \\times X \\to X,\\qquad (g,x) \\mapsto g\\!\\cdot\\!x
\\]
satisfying:
1. \\(e\\!\\cdot\\!x = x\\) for the identity \\(e \\in G\\).
2. \\((gh)\\!\\cdot\\!x = g \\!\\cdot\\! (h\\!\\cdot\\!x)\\) for all \\(g,h\\in G,\\;x\\in X\\).

*Usage*: Think of \\(G\\) as a symmetry group acting on a geometric shape \\(X\\).

---

## 2. Orbit

**Definition**  
The **orbit** of an element \\(x \\in X\\) under the action of \\(G\\) is
\\[
\\mathcal{O}_G(x) = \\{g\\!\\cdot\\!x \\mid g \\in G\\}.
\\]

It is the set of all points you can reach from \\(x\\) by applying group elements.

*Example*:  
- \\(G = \\mathbb{Z}_4\\) (cyclic of order 4) acting on the vertices of a square by rotation.  
- The orbit of a vertex is the set of all four vertices.

---

## 3. Isotropy Subgroup (Stabilizer)

**Definition**  
The **isotropy subgroup** (or stabilizer) of \\(x\\) is
\\[
\\operatorname{Stab}_G(x) = \\{g \\in G \\mid g\\!\\cdot\\!x = x\\}.
\\]

It consists of all group elements that leave \\(x\\) unchanged.

*Example*:  
- In the square rotation example, the stabilizer of a vertex is the identity element only, because only the 0° rotation fixes it.  
- For a reflection symmetry group acting on a line, the point at the center of reflection has the whole group as stabilizer.

**Orbit–Stabilizer Theorem**  
For a finite group \\(G\\) acting on \\(X\\):
\\[
|\\mathcal{O}_G(x)| \\,\\cdot\\, |\\operatorname{Stab}_G(x)| = |G|.
\\]
This links the size of an orbit to the size of its stabilizer.

---

## 4. Divisor (in Group Theory Context)

**Definition**  
A **divisor** of an element \\(g\\) in a group \\(G\\) is another element \\(d \\in G\\) such that \\(g = d^k\\) for some integer \\(k > 1\\).  
Informally, \\(d\\) generates a cyclic subgroup whose powers include \\(g\\).

*Example*:  
- In the additive group \\(\\mathbb{Z}_{12}\\), the element \\(8\\) has divisors \\(4\\) (since \\(4 \\times 2 = 8\\)) and \\(2\\) (since \\(2 \\times 4 = 8\\)).  
- In a multiplicative group modulo 13, \\(9\\) is a divisor of \\(3\\) because \\(9 = 3^2 \\pmod{13}\\).

**Usage**: Divisors help in factoring group elements and studying cyclic subgroups.

---

### Quick Recap

| Concept | What it describes | Key Formula / Property |
|---------|-------------------|------------------------|
| **Action** | How group elements transform points | \\(g\\!\\cdot\\!x\\) |
| **Orbit** | Set of reachable points from \\(x\\) | \\(\\mathcal{O}_G(x)\\) |
| **Isotropy subgroup (stabilizer)** | Elements fixing \\(x\\) | \\(\\operatorname{Stab}_G(x)\\) |
| **Divisor** | Element generating a power that equals another | \\(g = d^k\\) |

Feel free to ask if you want a concrete calculation or a deeper dive into any of these ideas!`,
		expectedTypes: [
			'Heading',
			'Blockquote',
			'HorizontalRule',
			'Heading',
			'Paragraph',
			'LatexBlock',
			'Paragraph',
			'OrderedList',
			'Paragraph',
			'HorizontalRule',
			'Heading',
			'Paragraph',
			'LatexBlock',
			'Paragraph',
			'Paragraph',
			'Paragraph',
			'LatexBlock',
			'Paragraph',
			'HorizontalRule',
			'Heading',
			'Paragraph',
			'LatexBlock',
			'Paragraph',
			'Paragraph',
			'Paragraph',
			'LatexBlock',
			'Paragraph',
			'HorizontalRule',
			'Heading',
			'Paragraph',
			'LatexBlock',
			'Paragraph',
			'Paragraph',
			'Paragraph',
			'LatexBlock',
			'Paragraph',
			'HorizontalRule',
			'Heading',
			'Table',
			'Paragraph'
		],
		deepExpectedPaths: [
			['Blockquote', 'Paragraph', 'Bold'],
			['Table', 'TableRow', 'TableCell', 'Bold'],
			['Table', 'TableRow', 'TableCell', 'LatexInline']
		]
	}
];

function randomChunks(text, seed) {
	const chunks = [];
	let pos = 0;
	let rng = seed;

	while (pos < text.length) {
		rng = (rng * 1103515245 + 12345) & 0x7fffffff;
		const chunkSize = Math.max(1, (rng % 8) + 1);
		const end = Math.min(pos + chunkSize, text.length);
		chunks.push(text.substring(pos, end));
		pos = end;
	}

	return chunks;
}

function generate() {
	const fixtures = templates.map((t) => ({
		...t,
		chunks: [
			randomChunks(t.markdown, 42),
			randomChunks(t.markdown, 123),
			randomChunks(t.markdown, 7)
		]
	}));

	const outPath = resolve(__dirname, 'codegen.json');
	writeFileSync(outPath, JSON.stringify(fixtures, null, '\t'));
	console.log(`Generated ${fixtures.length} fixtures to ${outPath}`);
}

generate();
