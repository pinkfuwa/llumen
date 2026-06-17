import type { Theme } from '$lib/preference/theme';
import type { BundledLanguage, BundledTheme } from './shiki.bundle';

const themeMap: Record<`${Theme['name']}-${Theme['dark']}`, BundledTheme> = {
	'dracula-true': 'dracula',
	'dracula-false': 'github-light',
	'llumen-true': 'github-dark',
	'llumen-false': 'github-light',
	'vitesse-true': 'vitesse-dark',
	'vitesse-false': 'vitesse-light'
};

const styleMap: Record<BundledTheme, string> = {
	'github-light': 'background-color:#fff;color:#24292e;caret-color:#24292e',
	'github-dark': 'background-color:#24292e;color:#e1e4e8;caret-color:#e1e4e8',
	'vitesse-dark': 'background-color:#121212;color:#dbd7caee;caret-color:#dbd7caee',
	'vitesse-light': 'background-color:#ffffff;color:#393a34;caret-color:#393a34',
	dracula: 'background-color:#282A36;color:#F8F8F2;caret-color:#F8F8F2'
};

export function getThemeName(x: Theme) {
	return themeMap[`${x.name}-${x.dark}`];
}

export function getThemeStyle(x: Theme) {
	return styleMap[getThemeName(x)];
}

export const bundle: Promise<typeof import('./shiki.bundle')> = import('./shiki.bundle');

const highlighter = bundle.then(async (bundle) => {
	return bundle.createHighlighter({
		themes: [],
		langs: []
	});
});

export async function getHighlighter(lang: string, theme: BundledTheme) {
	return highlighter.then(async (h) => {
		if (!h.getLoadedThemes().includes(theme)) {
			await h.loadTheme(theme);
		}
		if (!h.getLoadedLanguages().includes(lang)) {
			try {
				await h.loadLanguage(lang as BundledLanguage);
			} catch {}
		}
		return h;
	});
}
