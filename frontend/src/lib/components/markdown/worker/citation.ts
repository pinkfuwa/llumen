import type { MarkedExtension } from 'marked';

const citationRegex = /<citation[^>]*>([\s\S]*?)<\/citation>/;

const fieldRegex = /<(\w+)>([\s\S]*?)<\/\1>/g;

const Citation: MarkedExtension = {
	extensions: [
		{
			name: 'citation',
			level: 'block',
			tokenizer(src: string) {
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
						raw: match[0],
						...fields
					};
				}

				return undefined;
			}
		}
	]
};

export default Citation;
