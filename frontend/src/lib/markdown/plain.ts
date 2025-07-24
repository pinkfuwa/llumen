import { marked } from 'marked';
import { Octokit } from '@octokit/rest';
import { markedEmoji } from 'marked-emoji';
import markedShiki from 'marked-shiki';
import { codeToHtml } from 'shiki';

const octokit = new Octokit();
try {
	const emojis = await octokit.rest.emojis.get();

	marked.use(
		markedEmoji({
			emojis: emojis.data,
			renderer: (token) =>
				`<img alt="${token.name}" src="${token.emoji}" style="display: inline-block; height: 1em; width: 1em;">`
		})
	);
} catch (error) {
	console.warn(`error loading emojis: ${error}`);
}

marked.use(
	markedShiki({
		async highlight(code, lang) {
			return await codeToHtml(code, { lang, theme: 'min-light' });
		}
	})
);

const isFirefox = typeof navigator !== 'undefined' && navigator.userAgent.includes('Firefox');

export async function renderPlainMarkdown(content: string): Promise<string> {
	console.log(content);
	if (isFirefox) {
		content = content.replaceAll(/(^|\n)(\s*)\*\s*/g, '$1* ');
	}

	const html = marked.parse(content);
	return html instanceof Promise ? await html : html;
}
