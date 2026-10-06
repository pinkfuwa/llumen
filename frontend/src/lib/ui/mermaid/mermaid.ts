export const MERMAID_LANGUAGES = new Set([
	'mermaid',
	'graph',
	'flowchart',
	'sequence',
	'class',
	'state',
	'er',
	'xychart',
	'xychart-beta'
]);

export function isMermaidLanguage(lang: string | undefined): boolean {
	if (!lang) return false;
	return MERMAID_LANGUAGES.has(lang.toLowerCase());
}

export async function render(code: string): Promise<string> {
	const { renderMermaidSVG } = await import('beautiful-mermaid');
	const cleanCode = code.replaceAll(/^\s*style\s+\S+.*$/gm, '').trim();

	return renderMermaidSVG(cleanCode, {
		bg: 'var(--card)',
		fg: 'var(--foreground)',
		line: 'var(--muted-foreground)',
		accent: 'var(--primary)',
		muted: 'var(--muted-foreground)',
		surface: 'var(--secondary)',
		border: 'var(--mermaid-border)',
		font: 'ui-sans-serif, system-ui, sans-serif',
		transparent: true
	});
}
