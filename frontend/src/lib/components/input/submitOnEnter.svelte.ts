import { preference } from '$lib/preference/index.svelte';

type SubmitOnEnterOptions = {
	virtualKeyboard?: boolean;
};

export function shouldSubmitOnEnter(event: KeyboardEvent, options: SubmitOnEnterOptions = {}) {
	if (options.virtualKeyboard) return false;
	return event.key === 'Enter' && !event.shiftKey && preference.value.submit_on_enter === 'true';
}
