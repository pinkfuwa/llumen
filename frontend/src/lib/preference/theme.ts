export type Theme = 'light' | 'dark' | 'blue';

export interface ThemeStyle {
	light: string;
	dark: string;
	background: string;
	outline: string;
	hover: string;
	primary: string;
}

export function isLightTheme(theme: Theme) {
	return theme == 'light';
}

export function getTitleGrad(theme: Theme) {
	if (theme == 'blue') return 'from-slate-600 to-orange-600';
	if (theme == 'light') return 'from-slate-700 to-sky-500';
	return 'from-dark to-blue-600';
}

export function setTheme(theme: Theme) {
	document.body.setAttribute('data-theme', theme);
}
