export type Theme = 'light' | 'dark' | 'orange' | 'blue' | 'custom';

type PartialRecord<K extends keyof any, T> = {
	[P in K]?: T;
};

export interface ThemeStyle {
	light: string;
	dark: string;
	background: string;
	outline: string;
	hover: string;
	primary: string;
}

export async function setTheme(theme: Theme) {
	const themeMap: PartialRecord<Theme, ThemeStyle> = {
		light: {
			light: 'white',
			dark: 'black',
			background: '#f8f8f8',
			outline: '#ccc',
			hover: '#d6d6d6',
			primary: '#ebebeb'
		},
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

	let themeStyle = themeMap[theme];

	if (!themeStyle) {
		const res = await fetch('/customTheme.json');
		themeStyle = (await res.json()) as ThemeStyle;
	}

	const style = Object.entries(themeStyle)
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
