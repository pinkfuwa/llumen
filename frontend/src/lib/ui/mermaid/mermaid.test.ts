import { describe, test, expect } from 'vitest';
import { isMermaidLanguage, render } from './mermaid';

describe('Mermaid language detection', () => {
	test.each([
		'mermaid',
		'graph',
		'flowchart',
		'sequence',
		'class',
		'state',
		'er',
		'xychart',
		'xychart-beta',
		'GRAPH',
		'Flowchart',
		'MERMAID'
	])('recognizes %s', (language) => {
		expect(isMermaidLanguage(language)).toBe(true);
	});

	test.each([
		undefined,
		'',
		'javascript',
		'python',
		'html',
		'rust',
		'gantt',
		'pie',
		'journey',
		'git'
	])('leaves %s as source text', (language) => {
		expect(isMermaidLanguage(language)).toBe(false);
	});
});

describe('beautiful-mermaid rendering', () => {
	test.each([
		'flowchart TD\nA[Start] --> B[End]',
		'sequenceDiagram\nAlice->>Bob: Hello',
		'classDiagram\nAnimal <|-- Duck',
		'stateDiagram-v2\n[*] --> Idle\nIdle --> [*]',
		'erDiagram\nCUSTOMER ||--o{ ORDER : places',
		'xychart-beta\nx-axis [Jan, Feb]\ny-axis 0 --> 10\nbar [3, 7]'
	])('renders %s using live CSS colors', async (source) => {
		const container = document.createElement('div');
		container.innerHTML = await render(source);
		const svg = container.querySelector('svg');
		expect(svg).not.toBeNull();
		expect(svg!.style.getPropertyValue('--bg')).toBe('var(--card)');
		expect(svg!.style.getPropertyValue('--fg')).toBe('var(--foreground)');
		expect(svg!.style.getPropertyValue('--accent')).toBe('var(--primary)');
		expect(svg!.style.getPropertyValue('--border')).toBe('var(--mermaid-border)');
	});

	test('keeps label markup as text', async () => {
		const container = document.createElement('div');
		container.innerHTML = await render(
			'graph TD\nA["<script>alert(1)</script>"] --> B["<img src=x onerror=alert(1)>"]'
		);
		expect(container.querySelector('script, img, [onerror]')).toBeNull();
		expect(container.textContent).toContain('<script>alert(1)</script>');
	});

	test('rejects invalid diagrams', async () => {
		await expect(render('not a diagram')).rejects.toThrow();
	});

	test('ignores inline node colors so the app theme controls nodes', async () => {
		const svg = await render('graph TD\nA[Start] --> B[End]\nstyle A fill:#123456');
		expect(svg).toContain('Start');
		expect(svg).not.toContain('#123456');
	});
});
