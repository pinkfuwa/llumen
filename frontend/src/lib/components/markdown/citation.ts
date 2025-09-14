import { marked } from 'marked';

// Regex to match the full citation block
const citationRegex = /<citation[^>]*>([\s\S]*?)<\/citation>/;

// Regex to extract each inner field: <title>...</title>
const fieldRegex = /<(\w+)>([\s\S]*?)<\/\1>/g;

marked.use({
	extensions: [
		{
			name: 'citation',
			level: 'block',
			tokenizer(src: string) {
				// Check if input starts with <citation>...
				const match = citationRegex.exec(src);
				if (match && match.index === 0) {
					const content = match[1];

					const fields: Record<string, string> = {};
					let fieldMatch;

					while ((fieldMatch = fieldRegex.exec(content)) !== null) {
						const key = fieldMatch[1].toLowerCase();
						const value = fieldMatch[2].trim();
						if (value) fields[key] = value;
					}

					return {
						type: 'citation',
						raw: match[0], // full match (consumed)
						...fields
					};
				}

				return undefined; // no match, let marked continue
			}
		}
	]
});

export default function initCitation() {
	console.log('citation latex');
}
