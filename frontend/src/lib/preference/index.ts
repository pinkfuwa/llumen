import { derived, get, type Readable } from 'svelte/store';
import { APIFetch } from '../api/state/errorHandle';
import type {
	UserReadReq,
	UserPreference,
	UserReadResp,
	UserUpdateResp,
	UserUpdateReq
} from '../api/types';
import { setLocale } from './i18n';
import { localState, syncState, token } from '../store';
import type { Theme } from './theme';
import { setTheme } from './theme';
import { onDestroy } from 'svelte';

export { propToRune } from './mutate.svelte';

function defaultPreference(): Required<UserPreference> {
	const dark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
	return {
		theme: { name: 'llumen', dark, pattern: false },
		locale: navigator.language.includes('zh') ? 'zh-tw' : 'en',
		submit_on_enter: 'true'
	};
}

export const preference = syncState(
	'preference',
	defaultPreference(),
	{
		upload: async (preference) => {
			await APIFetch<UserUpdateResp, UserUpdateReq>('user/update', { preference });
		},
		download: () =>
			new Promise((resolve) => {
				let unsubscribe = token.subscribe((token) => {
					if (token) {
						APIFetch<UserReadResp, UserReadReq>('user/read', {}).then((remote) => {
							if (!remote) return;
							resolve(remote.preference as Required<UserPreference>);
							unsubscribe();
						});
					}
				});
			})
	},
	(data) => {
		if (typeof data.theme === 'string') return false;
		return true;
	}
);

export async function init() {
	const unsubscribers = [
		preference.subscribe((value) => {
			setTheme(value.theme as Theme);
			setLocale(value.locale as any);
		})
	];
	onDestroy(() => unsubscribers.forEach((un) => un()));
}

export const submitOnEnter = derived(preference, (x) => x.submit_on_enter);

export const locale = derived(preference, (x) => x.locale);

export const theme: Readable<Theme> = derived(preference, (x) => x.theme as Theme);

export const lastModel = localState<number | null>('lastModel', null);
