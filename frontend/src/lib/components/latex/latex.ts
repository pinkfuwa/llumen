let katexInstance: Promise<typeof import('katex')> = import('katex');

export async function toHtml(text: string, displayMode: boolean) {
	const katexModule = await katexInstance;
	const katex = katexModule.default;

	return katex.renderToString(text, {
		displayMode,
		output: 'html',
		throwOnError: false
	});
}
