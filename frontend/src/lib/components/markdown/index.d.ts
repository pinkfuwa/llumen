interface ASTNode {
	type: string;
	from?: number;
	to?: number;
	text?: string;
	children?: ASTNode[];
	citationData?: CitationData;
}

interface CitationData {
	title?: string;
	url?: string;
	favicon?: string;
	authoritative?: boolean;
	raw: string;
}
