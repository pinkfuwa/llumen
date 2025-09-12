import type { Highlighter, BundledLanguage, BundledTheme } from './shiki.bundle';

let highlighter: Promise<Highlighter> | undefined;
let loadLangPromises = new Map<string, Promise<void>>();

async function getHighlighter() {
	if (highlighter == undefined) {
		highlighter = import('./shiki.bundle').then((x) =>
			x.createHighlighter({
				themes: ['github-light', 'github-dark'],
				langs: []
			})
		);
	}

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

	let loadLangPromise = loadLangPromises.get(options.lang);
	if (loadLangPromise === undefined) {
		loadLangPromise = highlighterInstance.loadLanguage(options.lang as BundledLanguage);
		loadLangPromises.set(options.lang, loadLangPromise);
	}

	await loadLangPromise;

	const theme = options.isLight ? 'github-light' : 'github-dark';

	return highlighterInstance.codeToHtml(code, {
		lang: options.lang as BundledLanguage,
		theme: theme as BundledTheme
	});
}
