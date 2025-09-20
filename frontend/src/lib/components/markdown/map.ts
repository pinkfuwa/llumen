import { marked } from 'marked';

// Regex to match the full citation block
const mapRegex = /<map[^>]*>([\s\S]*?)<\/map>/;

// Regex to extract each inner field: <title>...</title>
const fieldRegex = /<(\w+)>([\s\S]*?)<\/\1>/g;

/*
<map>
    <marker>
        <latitude></latitude>
        <longtitude></longtitude>
        <displayName></displayName>
        <address></address>
        <rating></rating>
    </marker>
	<marker>
        <latitude></latitude>
        <longtitude></longtitude>
        <displayName></displayName>
        <address></address>
        <rating></rating>
    </marker>
</map>
*/

marked.use({
    extensions: [
        {
            name: 'map',
            level: 'block',
            tokenizer(src: string) {
                // Check if input starts with <citation>...
                const match = mapRegex.exec(src);
                if (match && match.index === 0) {
                    const content = match[1];

                    const markerRegex = /<marker>([\s\S]*?)<\/marker>/g;
                    let markerMatch;
                    const fields: Record<string, string>[] = [];

                    while ((markerMatch = markerRegex.exec(content)) !== null) {
                        const markerContent = markerMatch[1];
                        let fieldMatch;
                        let field: Record<string, string> = {};
                        while ((fieldMatch = fieldRegex.exec(markerContent)) !== null) {
                            const key = fieldMatch[1].toLowerCase();
                            const value = fieldMatch[2].trim();
                            if (value) field[key] = value;
                        }
                        fields.push(field);
                    }

                    return {
                        type: 'map',
                        raw: match[0], // full match (consumed)
                        markerList: fields
                    };
                }

                return undefined; // no match, let marked continue
            }
        }
    ]
});

export default function initMap() {
    console.log('Map');
}
