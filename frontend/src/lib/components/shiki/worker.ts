import type { ShikiWorkerRequest } from './types';
import type { BundledLanguage } from './shiki.bundle';
import { getHighlighter } from './shiki';

const loaded = new Set<string>();

self.onmessage = async (event: MessageEvent<ShikiWorkerRequest>) => {
	const { code, lang, theme } = event.data;

	try {
		let highlighter = await getHighlighter(lang, theme);

		if (!loaded.has(lang)) {
			await highlighter.loadLanguage(lang as BundledLanguage);
			loaded.add(lang);
		}

		const html = highlighter.codeToHtml(code, {
			lang: lang as BundledLanguage,
			theme: theme
		});

		self.postMessage({ html });
	} catch (error) {
		self.postMessage({ error });
	}
};
