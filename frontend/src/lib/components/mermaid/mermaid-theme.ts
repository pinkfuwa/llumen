export function getMermaidDarkMode(): boolean {
	return document.documentElement.getAttribute('data-dark') === 'true';
}

export function getMermaidThemeVariables(): Record<string, string> {
	const style = getComputedStyle(document.documentElement);

	function getVar(name: string): string {
		return style.getPropertyValue(`--${name}`).trim();
	}

	return {
		background: getVar('background') || '#050505',
		primaryColor: getVar('card') || '#1c1c1c',
		primaryTextColor: getVar('foreground') || '#eeeeee',
		primaryBorderColor: getVar('primary') || '#ec5b2b',
		lineColor: getVar('muted-foreground') || '#808080',
		secondaryColor: getVar('accent') || '#1c1c1c',
		secondaryBorderColor: getVar('border') || '#434343',
		tertiaryColor: getVar('secondary') || '#141414',
		tertiaryBorderColor: getVar('border') || '#434343',
		fontFamily: 'ui-sans-serif, system-ui, sans-serif'
	};
}
