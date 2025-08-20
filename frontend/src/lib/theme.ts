export type Theme = 'light' | 'dark' | 'orange';
export function setTheme(theme: Theme) {
	const themeMap: Record<Theme, {}> = {
		light: {},
		dark: {
			light: 'black',
			dark: 'white',
			background: '#292827',
			outline: '#ccc',
			hover: '#706d69',
			primary: '#383735'
		},
		orange: {
			light: '#fcfbf0',
			dark: 'black',
			background: '#fdfae6',
			outline: '#ccc',
			hover: '#ffd25e',
			primary: '#fff0ab'
		}
	};

	const style = Object.entries(themeMap[theme])
		.map(([name, val]) => `--color-${name}: ${val};`)
		.join('');

	window.document.body.style.cssText = style;
}

export function isLightTheme(theme: Theme) {
	return theme == 'light' || theme == 'orange';
}

export function getTitleGrad(theme: Theme) {
	if (theme == 'orange') return 'from-slate-600 to-orange-600';
	if (theme == 'light') return 'from-slate-700 to-sky-500';
	return 'from-dark to-blue-600';
}
