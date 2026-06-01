import type { Theme } from '$lib/preference/theme';
import type { BundledTheme } from './shiki.bundle';

export function getThemeName(x: Theme) {
	const themeMap: Record<`${Theme['name']}-${Theme['dark']}`, BundledTheme> = {
		'dracula-true': 'dracula',
		'dracula-false': 'github-light',
		'llumen-true': 'github-dark',
		'llumen-false': 'github-light',
		'vitesse-true': 'vitesse-dark',
		'vitesse-false': 'vitesse-light'
	};
	return themeMap[`${x.name}-${x.dark}`];
}

export function getThemeStyle(x: Theme) {
	const styleMap: Record<BundledTheme, string> = {
		'github-light': 'background-color:#fff;color:#24292e',
		'github-dark': 'background-color:#24292e;color:#e1e4e8',
		'vitesse-dark': 'background-color:#121212;color:#dbd7caee',
		'vitesse-light': 'background-color:#ffffff;color:#393a34',
		dracula: 'background-color:#282A36;color:#F8F8F2'
	};
	return styleMap[getThemeName(x)];
}
