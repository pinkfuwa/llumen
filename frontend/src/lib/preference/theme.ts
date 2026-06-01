export type Theme = {
	name: 'dracula' | 'llumen' | 'vitesse';
	dark: boolean;
	pattern: boolean;
};

export function setTheme(theme: Theme) {
	const { name, dark, pattern } = theme;
	document.documentElement.setAttribute('data-theme', name);
	document.documentElement.setAttribute('data-dark', dark ? 'true' : 'false');
	document.documentElement.setAttribute('data-pattern', pattern ? 'true' : 'false');

	requestAnimationFrame(() => {
		const meta = document.querySelector('meta[name="theme-color"]')!;
		const style = window.getComputedStyle(document.documentElement);
		const color = style.getPropertyValue('--color-surface-base');
		meta.setAttribute('content', color);
	});
}
