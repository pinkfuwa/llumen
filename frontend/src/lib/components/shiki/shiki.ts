export function getThemeName(isLight: boolean) {
	return isLight ? 'github-light' : 'github-dark';
}

export function getThemeStyle(isLight: boolean) {
	return isLight ? 'background-color:#fff;color:#24292e' : 'background-color:#24292e;color:#e1e4e8';
}
