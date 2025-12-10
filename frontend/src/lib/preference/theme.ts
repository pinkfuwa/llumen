export type Theme = 'light' | 'dark' | 'blue' | 'light-pattern' | 'dark-pattern';

export interface ThemeStyle {
	light: string;
	dark: string;
	background: string;
	outline: string;
	hover: string;
	primary: string;
}

export function isLightTheme(theme: Theme) {
	return theme == 'light' || theme == 'light-pattern';
}

export function setTheme(theme: Theme) {
	document.documentElement.setAttribute('data-theme', theme);

	requestAnimationFrame(() => {
		const meta = document.querySelector('meta[name="theme-color"]')!;
		const style = window.getComputedStyle(document.documentElement);
		const color = style.getPropertyValue('--color-login-bg');
		meta.setAttribute('content', color);
	});
}
