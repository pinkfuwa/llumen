/**
 * Citation parser for extracting structured data from citation XML blocks.
 */

export interface CitationData {
	title?: string;
	url?: string;
	favicon?: string;
	authoritative?: boolean;
	raw: string;
}

/**
 * Parse a citation block and extract structured data.
 *
 * Expected format:
 * <citation>
 *     <title>Title of the source</title>
 *     <url>Full URL</url>
 *     <favicon>Favicon URL</favicon>
 *     <authoritative />
 * </citation>
 */
export function parseCitation(text: string): CitationData {
	const raw = text;
	const data: CitationData = { raw };

	// Extract title
	const titleMatch = text.match(/<title>(.*?)<\/title>/s);
	if (titleMatch) {
		data.title = titleMatch[1].trim();
	}

	// Extract url
	const urlMatch = text.match(/<url>(.*?)<\/url>/s);
	if (urlMatch) {
		data.url = urlMatch[1].trim();
	}

	// Extract favicon
	const faviconMatch = text.match(/<favicon>(.*?)<\/favicon>/s);
	if (faviconMatch) {
		data.favicon = faviconMatch[1].trim();
	}

	// Check for authoritative tag (self-closing or empty)
	const authoritativeMatch = text.match(/<authoritative\s*\/?>/);
	if (authoritativeMatch) {
		data.authoritative = true;
	}

	return data;
}

/**
 * Check if a text block is a citation block.
 */
export function isCitationBlock(text: string): boolean {
	return text.trimStart().startsWith('<citation>') && text.includes('</citation>');
}
