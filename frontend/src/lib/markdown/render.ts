import { marked } from 'marked';
import DOMPurify from 'isomorphic-dompurify';
import { Octokit } from '@octokit/rest';
import { markedEmoji } from 'marked-emoji';
import markedShiki from 'marked-shiki';
import { codeToHtml } from 'shiki';

const octokit = new Octokit();
const emojis = await octokit.rest.emojis.get();

marked
	.use(
		markedEmoji({
			emojis: emojis.data,
			renderer: (token) =>
				`<img alt="${token.name}" src="${token.emoji}" style="display: inline-block; height: 1em; width: 1em;">`
		})
	)
	.use(
		markedShiki({
			async highlight(code, lang) {
				return await codeToHtml(code, { lang, theme: 'min-light' });
			}
		})
	);

function applyStyle(htmlString: string): string {
	const parser = new DOMParser();
	const doc = parser.parseFromString(htmlString, 'text/html');

	const styleMap = new Map<string, string>([
		['h1', 'text-2xl font-bold'],
		['h2', 'text-xl font-bold'],
		['h3', 'text-lg font-bold'],
		['pre', 'rounded-md border border-outline p-2 border-radius-md overflow-x-auto'],
		['p', 'items-center'],
		['ul', 'list-disc ml-6']
	]);

	styleMap.forEach((classValue, tagName) => {
		const elements = doc.querySelectorAll(tagName);
		elements.forEach((element) => {
			const currentClass = element.getAttribute('class') || '';
			element.setAttribute('class', `${currentClass} ${classValue}`.trim());
		});
	});

	return doc.documentElement.outerHTML;
}

const isFirefox = typeof navigator !== 'undefined' && navigator.userAgent.includes('Firefox');

export async function render(content: string): Promise<string> {
	if (isFirefox) {
		content = content.replace(/(^|\n)(\s*)\*\s*/g, '$1* ');
	}

	const html = marked.parse(content);
	return applyStyle(DOMPurify.sanitize(html instanceof Promise ? await html : html));
}
