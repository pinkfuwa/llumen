import DOMPurify from 'isomorphic-dompurify';
import { marked } from 'marked';
import { Octokit } from '@octokit/rest';
import { markedEmoji } from 'marked-emoji';
import markedShiki from 'marked-shiki';
import { codeToHtml } from 'shiki';
import firefox from './firefox';
import style from './style';

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

marked.use({ hooks: { postprocess: (html) => DOMPurify.sanitize(html) } });

marked.use(firefox);
marked.use(style);

/**
 * Direct render of markdown content
 * @param content
 * @returns
 */
export async function render(content: string): Promise<string> {
	const html = marked.parse(content);
	return html instanceof Promise ? await html : html;
}

render(['# test code block', '```javascript', "console.log('hello world');", '```'].join('\n'));
