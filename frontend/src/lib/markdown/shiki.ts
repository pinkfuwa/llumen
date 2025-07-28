import markedShiki from 'marked-shiki';
import { codeToHtml } from 'shiki';
import type { BundledTheme, StringLiteralUnion, ThemeRegistrationAny } from 'shiki/types';

const copySVG = (classes: string) =>
	`<svg class="${classes}" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-clipboard-copy-icon lucide-clipboard-copy"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/><path d="M16 4h2a2 2 0 0 1 2 2v4"/><path d="M21 14H11"/><path d="m15 10-4 4 4 4"/></svg>`;

export default function (theme: ThemeRegistrationAny | StringLiteralUnion<BundledTheme, string>) {
	return markedShiki({
		async highlight(code, lang) {
			return await codeToHtml(code, { lang, theme });
		},
		container: `<div class="relative group ll-codeblock-copy">${copySVG('absolute top-0 right-0 z-10 p-2 m-1 rounded-md h-10 w-10 hidden group-hover:block bg-light hover:bg-hover bg-background ll-codeblock-svg')}<div class="rounded-md border border-outline border-radius-md ll-codeblock-code">%s</div></div>`
	});
}
