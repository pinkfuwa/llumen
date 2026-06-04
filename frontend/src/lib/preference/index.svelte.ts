import { APIFetch } from '../api/errorHandle.svelte';
import type {
	UserReadReq,
	UserPreference,
	UserReadResp,
	UserUpdateResp,
	UserUpdateReq
} from '../api/types';
import { setLocale } from '$lib/paraglide/runtime';
import { localState, token } from '../store.svelte';
import type { Theme } from './theme';
import { setTheme } from './theme';

function defaultPreference(): Required<UserPreference> {
	const dark = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false;
	return {
		theme: { name: 'llumen', dark, pattern: true },
		locale: navigator.language.includes('zh') ? 'zh-tw' : 'en',
		submit_on_enter: 'true'
	};
}

const preferenceChecker = (data: Required<UserPreference>) => typeof data.theme !== 'string';

export const preference = localState<Required<UserPreference>>('preference', {
	defaultValue: defaultPreference,
	checker: preferenceChecker,
	syncer: {
		upload: async (p) => {
			await APIFetch<UserUpdateResp, UserUpdateReq>('user/update', { preference: p });
		},
		download: async () => {
			const remote = await APIFetch<UserReadResp, UserReadReq>('user/read', {});
			if (!remote) return null;
			return remote.preference as Required<UserPreference>;
		}
	}
});

export const lastModel = localState<number | null>('lastModel', {
	defaultValue: () => null
});

$effect.root(() => {
	$effect(() => {
		setTheme(preference.value.theme as Theme);
		setLocale(preference.value.locale as any);
	});
	return () => {};
});

$effect.root(() => {
	$effect(() => {
		if (token.value) preference.sync();
	});
	return () => {};
});
