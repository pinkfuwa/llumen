function latexTrim(i: string) {
	return i
		.replace(/^\$*\s*/gm, '')
		.replace(/\s*\$*$/gm, '')
		.replace(/^\\\[\s*/gm, '')
		.replace(/\s*\\\]$/gm, '')
		.replace(/^\\\(\s*/gm, '')
		.replace(/\s*\\\)$/gm, '');
}

let texzillaInstance: Promise<typeof import('texzilla')> | null = null;

export async function toHtml(text: string, displayMode: boolean) {
	if (texzillaInstance === null) texzillaInstance = import('texzilla');

	const texzilla = await texzillaInstance;

	return texzilla.toMathMLString(latexTrim(text), {
		displayMode,
		displaystyle: displayMode
	});
}
