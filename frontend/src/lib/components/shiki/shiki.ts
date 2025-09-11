import type { Highlighter, BundledLanguage, BundledTheme } from './shiki.bundle';

let highlighter: Highlighter | null = null;

async function getHighlighter() {
	if (highlighter != null) return highlighter;
	const shiki = await import('./shiki.bundle');

	highlighter = await shiki.createHighlighter({
		themes: [],
		langs: []
	});

	return highlighter;
}

export function getThemeName(isLight: boolean) {
	return isLight ? 'github-light' : 'github-dark';
}

export function getThemeStyle(isLight: boolean) {
	return isLight ? 'background-color:#fff;color:#24292e' : 'background-color:#24292e;color:#e1e4e8';
}

export async function codeToHtml(
	code: string,
	options: { isLight: boolean; lang: string }
): Promise<string> {
	const highlighterInstance = await getHighlighter();

	const theme = options.isLight ? 'github-light' : 'github-dark';

	return highlighterInstance.codeToHtml(code, {
		lang: options.lang as BundledLanguage,
		theme: theme as BundledTheme
	});
}
