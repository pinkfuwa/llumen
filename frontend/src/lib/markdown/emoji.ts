import { Octokit } from '@octokit/rest';
import type { MarkedExtension } from 'marked';
import { markedEmoji } from 'marked-emoji';

let emojiExt: MarkedExtension<string, string> = {};
try {
	const octokit = new Octokit();
	const emojis = await octokit.rest.emojis.get();

	emojiExt = markedEmoji({
		emojis: emojis.data,
		renderer: (token) =>
			`<img alt="${token.name}" src="${token.emoji}" style="display: inline-block; height: 1em; width: 1em;">`
	});
} catch (error) {
	console.warn(`error loading emojis: ${error}`);
}

export default emojiExt;
