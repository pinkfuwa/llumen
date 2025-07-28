function applyStyle(htmlString: string): string {
	const parser = new DOMParser();
	const doc = parser.parseFromString(htmlString, 'text/html');

	const styleMap = new Map<string, string>([
		['h1', 'text-2xl font-bold'],
		['h2', 'text-xl font-bold'],
		['h3', 'text-lg font-bold'],
		['p', 'items-center'],
		['ul', 'list-disc ml-6'],
		['hr', 'border-outline'],
		['ol', 'list-decimal ml-6'],
		['p > code', 'bg-background p-1 rounded-md'],
		['pre', 'p-3 overflow-x-auto']
	]);

	styleMap.forEach((classValue, cssQuery) => {
		const elements = doc.querySelectorAll(cssQuery);
		elements.forEach((element) => {
			const currentClass = element.getAttribute('class') || '';
			element.setAttribute('class', `${currentClass} ${classValue}`.trim());
		});
	});

	doc.querySelectorAll('div.ll-codeblock-copy').forEach((element) => {
		let svg = element.querySelector('svg.ll-codeblock-svg')!;
		let code = element.querySelector('div.ll-codeblock-code')!;
		code.setAttribute('id', `codeblock-source-${crypto.randomUUID()}`);
		svg.setAttribute(
			'onclick',
			`{let code = document.getElementById('${code.id}'); navigator.clipboard.writeText(code.textContent);}`
		);
	});

	return doc.documentElement.outerHTML;
}

export default {
	hooks: {
		postprocess: applyStyle
	}
};
