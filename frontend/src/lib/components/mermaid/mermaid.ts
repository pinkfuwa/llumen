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

const lightTheme = {
	primaryColor: '#3b82f6',
	primaryTextColor: '#1f2937',
	primaryBorderColor: '#3b82f6',
	lineColor: '#6b7280',
	secondaryColor: '#e5e7eb',
	tertiaryColor: '#f3f4f6',
	background: '#ffffff',
	mainBkg: '#f3f4f6',
	nodeBorder: '#3b82f6',
	clusterBkg: '#f3f4f6',
	clusterBorder: '#e5e7eb',
	titleColor: '#1f2937',
	edgeLabelBackground: '#ffffff'
};

const darkTheme = {
	primaryColor: '#60a5fa',
	primaryTextColor: '#f3f4f6',
	primaryBorderColor: '#60a5fa',
	lineColor: '#9ca3af',
	secondaryColor: '#374151',
	tertiaryColor: '#1f2937',
	background: '#171717',
	mainBkg: '#1f2937',
	nodeBorder: '#60a5fa',
	clusterBkg: '#1f2937',
	clusterBorder: '#374151',
	titleColor: '#f3f4f6',
	edgeLabelBackground: '#171717'
};

export async function render(code: string, isDark: boolean = false): Promise<string> {
	if (mermaidInstance === null) {
		mermaidInstance = import('mermaid');
	}

	const mermaidModule = await mermaidInstance;
	const id = `mermaid-${++idCounter}`;
	const theme = isDark ? darkTheme : lightTheme;

	mermaidModule.default.initialize({
		startOnLoad: false,
		theme: 'base',
		themeVariables: theme
	});

	try {
		const { svg } = await mermaidModule.default.render(id, code);
		return svg;
	} catch {
		throw new Error('Failed to render mermaid diagram');
	}
}
