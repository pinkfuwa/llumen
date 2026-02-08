import { get } from 'svelte/store';
import { submitOnEnter } from '$lib/preference';

type SubmitOnEnterOptions = {
	virtualKeyboard?: boolean;
};

export function shouldSubmitOnEnter(event: KeyboardEvent, options: SubmitOnEnterOptions = {}) {
	if (options.virtualKeyboard) return false;
	return event.key === 'Enter' && !event.shiftKey && get(submitOnEnter) === 'true';
}
