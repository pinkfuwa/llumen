function latexTrim(i: string) {
	return i
		.replace(/^\$*\s*/gm, '')
		.replace(/\s*\$*$/gm, '')
		.replace(/^\\\[\s*/gm, '')
		.replace(/\s*\\\]$/gm, '')
		.replace(/^\\\(\s*/gm, '')
		.replace(/\s*\\\)$/gm, '');
}

let temmlInstance: Promise<typeof import('temml')> | null = null;

export async function toHtml(text: string, displayMode: boolean) {
	if (temmlInstance === null) temmlInstance = import('temml');

	const temmlModule = await temmlInstance;
	const temml = temmlModule.default;

	return temml.renderToString(latexTrim(text), {
		displayMode,
		throwOnError: false
	});
}
