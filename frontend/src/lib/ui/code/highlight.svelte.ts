import type { Attachment } from 'svelte/attachments';
import { highlightQueue } from './client';
import { codeLanguage } from './languages';
import { paintTokens } from './paint';
import type { CodeSnapshot } from './types';

export function syntaxHighlight(snapshot: () => CodeSnapshot): Attachment<HTMLElement> {
	return (element) => {
		if (!globalThis.CSS?.highlights || typeof Highlight === 'undefined') return;
		const queue = highlightQueue();
		const owner = {};
		let revision = 0;
		let latest: CodeSnapshot;
		let timer: ReturnType<typeof setTimeout> | undefined;
		let clearPaint = () => {};

		function submit() {
			timer = undefined;
			const language = codeLanguage(latest.lang);
			if (!language || language === 'plain' || !latest.text) return;
			const currentRevision = revision;
			queue.request(owner, latest.text, language, (tokens) => {
				if (revision !== currentRevision) return;
				clearPaint();
				clearPaint = paintTokens(element, tokens);
			});
		}

		$effect(() => {
			latest = snapshot();
			revision++;
			clearPaint();
			queue.cancel(owner);
			if (!latest.incremental) {
				clearTimeout(timer);
				submit();
			} else if (!timer) {
				const delay = latest.text.length > 262144 ? 1000 : latest.text.length > 65536 ? 250 : 100;
				timer = setTimeout(submit, delay);
			}
		});

		return () => {
			revision++;
			clearTimeout(timer);
			queue.cancel(owner);
			clearPaint();
		};
	};
}
