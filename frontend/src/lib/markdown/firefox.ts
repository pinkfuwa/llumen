class CodeBlockState {
	private content: string[] = [];
	private codeblock: string[] = [];
	private last?: { language: string; indent: number };
	private processLine(line: string) {
		const codeblockSyntax = line.trimStart().startsWith('```');
		switch (codeblockSyntax.toString() + (this.last !== undefined).toString()) {
			case 'truefalse':
				// start of code block
				this.last = {
					language: line.trimStart().slice(3),
					indent: line.length - line.trimStart().length
				};
				break;
			case 'falsefalse':
				this.content.push(line);
				break;
			case 'falsetrue':
				// inside code block
				this.codeblock.push(line);
				break;
			case 'truetrue':
				// end of code block
				this.flushCodeblock();
				this.last = undefined;
				break;
		}
	}
	private flushCodeblock() {
		// find min indent
		const minIndent = Math.min(
			...this.codeblock
				.filter((line) => line.trim().length > 0)
				.map((line) => line.match(/^\s*/)?.[0].length || 0)
		);
		if (minIndent > 0) {
			this.content.push(
				`\`\`\`${this.last!.language}\n${this.codeblock.map((line) => line.slice(minIndent)).join('\n')}\n\`\`\``
			);
		} else {
			this.content.push(`\`\`\`${this.last!.language}\n${this.codeblock.join('\n')}\n\`\`\``);
		}
		this.codeblock = [];
	}
	public process(content: string) {
		content.split('\n').forEach((line) => {
			this.processLine(line);
		});
		// if (this.last) this.flushCodeblock();
		return this.content.join('\n');
	}
}

const isFirefox = typeof navigator !== 'undefined' && navigator.userAgent.includes('Firefox');

function firefoxFix(content: string): string {
	if (isFirefox) {
		content = content.replaceAll(/(^|\n)(\s*)\*\s*/g, '$1* ');
		content = content.replaceAll(/(^|\n)(\s*)(\d+\.)\s*/g, '$1$2$3 ');
		content = new CodeBlockState().process(content);
	}

	return content;
}

export default {
	hooks: {
		preprocess: isFirefox ? firefoxFix : (content: string) => content
	}
};
