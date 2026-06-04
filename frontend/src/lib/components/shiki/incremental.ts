import type { ThemedToken } from 'shiki';

export function htmlEscape(s: string): string {
	return s
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

export function buildTokenHtml(token: ThemedToken): string {
	const styles: string[] = [];
	if (token.color) styles.push(`color:${token.color}`);
	if (token.fontStyle != null) {
		if (token.fontStyle & 1) styles.push('font-style:italic');
		if (token.fontStyle & 2) styles.push('font-weight:bold');
		if (token.fontStyle & 4) styles.push('text-decoration:underline');
	}
	const content = htmlEscape(token.content);
	if (styles.length > 0) {
		return `<span style="${styles.join(';')}">${content}</span>`;
	}
	return `<span>${content}</span>`;
}

export function tokensToHtml(tokens: ThemedToken[]): string {
	let html = '';
	for (const t of tokens) {
		html += buildTokenHtml(t);
	}
	return html;
}
