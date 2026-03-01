import './mermaid.css';

let mermaidInstance: Promise<typeof import('mermaid')> | null = null;

export const MERMAID_LANGUAGES = new Set([
	'mermaid',
	'graph',
	'flowchart',
	'sequence',
	'class',
	'state',
	'er',
	'gantt',
	'pie',
	'journey',
	'git'
]);

export function isMermaidLanguage(lang: string | undefined): boolean {
	if (!lang) return false;
	return MERMAID_LANGUAGES.has(lang.toLowerCase());
}

let idCounter = 0;

export async function render(code: string): Promise<string> {
	if (mermaidInstance === null) {
		mermaidInstance = import('mermaid');
	}

	const mermaidModule = await mermaidInstance;
	const id = `mermaid-${++idCounter}`;

	mermaidModule.default.initialize({
		startOnLoad: false,
		theme: 'base'
	});

	const cleanCode = code.replaceAll(/^\s*style\s+\S+.*$/gm, '').trim();

	try {
		const { svg } = await mermaidModule.default.render(id, cleanCode);
		return svg;
	} catch {
		throw new Error('Failed to render mermaid diagram');
	}
}
