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

/**
 * Split a text block containing multiple citation blocks into individual citations.
 * Returns an array of citation text blocks with their positions.
 */
export function splitCitations(
	text: string,
	startOffset: number = 0
): Array<{ text: string; from: number; to: number }> {
	const citations: Array<{ text: string; from: number; to: number }> = [];
	let pos = 0;

	while (pos < text.length) {
		const openIndex = text.indexOf('<citation>', pos);
		if (openIndex === -1) break;

		const closeIndex = text.indexOf('</citation>', openIndex + 10);
		if (closeIndex === -1) {
			// Unclosed citation, include from opening to end
			citations.push({
				text: text.slice(openIndex),
				from: startOffset + openIndex,
				to: startOffset + text.length
			});
			break;
		}

		const blockEnd = closeIndex + 11; // Length of '</citation>'
		citations.push({
			text: text.slice(openIndex, blockEnd),
			from: startOffset + openIndex,
			to: startOffset + blockEnd
		});
		pos = blockEnd;
	}

	return citations;
}
