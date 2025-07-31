export function setTheme(theme: 'light' | 'dark') {
	const themeMap: Record<'light' | 'dark', {}> = {
		light: {},
		dark: {
			light: 'black',
			dark: 'white',
			background: '#292827',
			outline: '#ccc',
			hover: '#706d69',
			primary: '#ce5f27'
		}
	};

	const style = Object.entries(themeMap[theme])
		.map(([name, val]) => `--color-${name}: ${val};`)
		.join('');

	window.document.body.style.cssText = style;
}
