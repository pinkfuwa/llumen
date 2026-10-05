import type { Attachment } from 'svelte/attachments';
import { highlightQueue } from './client';
import { paintTokens } from './paint';
import { CodeSnapshots } from './snapshots';
import type { CodeSnapshot } from './types';

export function syntaxHighlight(snapshot: () => CodeSnapshot): Attachment<HTMLElement> {
	return (element) => {
		if (!globalThis.CSS?.highlights || typeof Highlight === 'undefined') return;
		const snapshots = new CodeSnapshots(highlightQueue(), (tokens) => paintTokens(element, tokens));
		$effect(() => snapshots.update(snapshot()));
		return () => snapshots.destroy();
	};
}
