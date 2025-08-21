export type Theme = 'light' | 'dark' | 'orange' | 'blue';
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
			light: '#f7f7f5',
			dark: 'black',
			background: '#fdfae6',
			outline: '#ccc',
			hover: '#ffd25e',
			primary: '#fff0ab'
		},
		blue: {
			light: '#12395c',
			dark: 'white',
			background: '#0d2840',
			outline: '#ccc',
			hover: '#3f7eb5',
			primary: '#1378d6'
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
	if (theme == 'blue') return 'from-white to-sky-500';
	if (theme == 'light') return 'from-slate-700 to-sky-500';
	return 'from-dark to-blue-600';
}
