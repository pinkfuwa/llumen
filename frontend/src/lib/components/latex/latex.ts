function latexTrim(i: string) {
	return i
		.replace(/^\$\$\s*/gm, '')
		.replace(/\s*\$\$$/gm, '')
		.replace(/^\\\[\s*/gm, '')
		.replace(/\s*\\\]$/gm, '')
		.replace(/^\\\(\s*/gm, '')
		.replace(/\s*\\\)$/gm, '');
}

let katexInstance: Promise<typeof import('katex')> | null = null;

export async function toHtml(text: string, displayMode: boolean) {
	if (katexInstance === null) katexInstance = import('katex');

	const katex = await katexInstance;

	return katex.renderToString(latexTrim(text), {
		displayMode,
		output: 'mathml',
		throwOnError: false
	});
}
