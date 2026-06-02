import { token } from './store.svelte';

type ErrorDetail = {
	error: string;
	reason?: string;
};

export const error = $state<{ val: ErrorDetail | null }>({ val: null });

export function displayError(errorMsg: string, reason?: string) {
	error.val = { error: errorMsg, reason };
}

$effect.root(() => {
	if (error.val && error.val.error == 'malformed_token') token.value = undefined;
});
