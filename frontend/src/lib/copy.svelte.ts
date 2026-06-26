export const copyCounter = $state({ val: 0 });

export function copy(text: string) {
	navigator.clipboard.writeText(text);
	copyCounter.val += 1;
}

export function prompt() {
	copyCounter.val += 1;
}
