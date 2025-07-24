import DOMPurify from 'isomorphic-dompurify';
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

function dedent(block: string): string {
	const lines = block.split('\n');
	if (lines.length <= 1) return block.trim();

	const nonEmptyLines = lines.filter((line) => line.trim().length > 0);
	if (nonEmptyLines.length === 0) return block;

	const indentLengths = nonEmptyLines.map((line) => line.match(/^\s*/)?.[0].length || 0);
	const minIndent = Math.min(...indentLengths.filter((length) => length > 0)); // Minimum > 0

	if (minIndent === 0) return block;

	return lines
		.map((line) => line.slice(minIndent))
		.join('\n')
		.trim();
}

function processCodeBlocks(content: string): string {
	const lines = content.split('\n');
	const outputLines: string[] = [];
	let inCodeBlock = false;
	let codeContentLines: string[] = [];

	for (const line of lines) {
		if (!inCodeBlock) {
			if (line.trim().startsWith('```')) {
				outputLines.push(line.trimStart());
				inCodeBlock = true;
				codeContentLines = [];
			} else {
				outputLines.push(line);
			}
		} else {
			if (line.trim().startsWith('```')) {
				const dedentedCode = dedent(codeContentLines.join('\n'));
				outputLines.push(dedentedCode);
				outputLines.push(line.trimStart());
				inCodeBlock = false;
			} else {
				codeContentLines.push(line);
			}
		}
	}

	return outputLines.join('\n');
}

const isFirefox = typeof navigator !== 'undefined' && navigator.userAgent.includes('Firefox');

export async function renderPlainMarkdown(content: string): Promise<string> {
	if (isFirefox) {
		content = content.replaceAll(/(^|\n)(\s*)\*\s*/g, '$1* ');
		content = content.replaceAll(/(^|\n)(\s*)(\d+\.)\s*/g, '$1$2$3 ');
		content = processCodeBlocks(content);
	}

	const html = marked.parse(content);
	return html instanceof Promise ? await html : html;
}

function applyStyle(htmlString: string): string {
	const parser = new DOMParser();
	const doc = parser.parseFromString(htmlString, 'text/html');

	const styleMap = new Map<string, string>([
		['h1', 'text-2xl font-bold'],
		['h2', 'text-xl font-bold'],
		['h3', 'text-lg font-bold'],
		['pre', 'rounded-md border border-outline p-2 border-radius-md overflow-x-auto'],
		['p', 'items-center'],
		['ul', 'list-disc ml-6'],
		['hr', 'border-outline'],
		['ol', 'list-decimal ml-6']
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

export async function render(html: string): Promise<string> {
	const unsafeHTML = await renderPlainMarkdown(html);
	const unstyledHTML = DOMPurify.sanitize(unsafeHTML);
	return applyStyle(unstyledHTML);
}

render(`# preheat markdown`);
