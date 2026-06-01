export type Theme =
	| 'light'
	| 'light-pattern'
	| 'dark'
	| 'dark-pattern'
	| 'blue'
	| 'solarized-light'
	| 'solarized-dark'
	| 'dracula'
	| 'nord';

export function isLightTheme(theme: Theme) {
	return theme == 'light' || theme == 'light-pattern' || theme == 'solarized-light';
}

export function setTheme(theme: Theme) {
	document.documentElement.setAttribute('data-theme', theme);

	requestAnimationFrame(() => {
		const meta = document.querySelector('meta[name="theme-color"]')!;
		const style = window.getComputedStyle(document.documentElement);
		const color = style.getPropertyValue('--color-surface-base');
		meta.setAttribute('content', color);
	});
}
