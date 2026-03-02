import { writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

/** @type {{ name: string; markdown: string; expectedTypes: string[] }[]} */
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
