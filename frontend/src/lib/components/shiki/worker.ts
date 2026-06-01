import type { ShikiWorkerRequest } from './types';
import type { BundledLanguage, BundledTheme } from './shiki.bundle';
import { createHighlighter } from './shiki.bundle';

let highlighter = await createHighlighter({
	themes: ['github-light', 'github-dark', 'vitesse-dark', 'vitesse-light', 'dracula'],
	langs: []
});

const loaded = new Set<string>();

self.onmessage = async (event: MessageEvent<ShikiWorkerRequest>) => {
	const { code, lang, theme } = event.data;

	try {
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
