import { copy } from '$lib/copy.svelte';

export function clickOnCopy(element: () => HTMLElement | null, content: () => string) {
	let blockReleaseCopy = $state(false);
	$effect(() => {
		const e = element();
		if (!e) return;
		function oncopy() {
			blockReleaseCopy = true;
		}
		function onclick() {
			if (!blockReleaseCopy) copy(content());
			blockReleaseCopy = false;
		}
		e.addEventListener('copy', oncopy);
		e.addEventListener('click', onclick);
		return () => {
			e.removeEventListener('copy', oncopy);
			e.removeEventListener('click', onclick);
		};
	});
}
