import { derived, get } from 'svelte/store';
import { APIFetch } from '../api/state/errorHandle';
import type {
	UserReadReq,
	UserPreference,
	UserReadResp,
	UserUpdateResp,
	UserUpdateReq
} from '../api/types';
import { setLocale } from './i18n';
import { localState, token } from '../store';
import { getTitleGrad, setTheme } from './theme';
import { onDestroy } from 'svelte';
import { isLightTheme as isLightThemeFn } from './theme';

function getRemotePreference() {
	return APIFetch<UserReadResp, UserReadReq>('user/read', {});
}

function setRemotePreference(preference: UserPreference) {
	return APIFetch<UserUpdateResp, UserUpdateReq>('user/update', { preference });
}

function defaultPreference(): Required<UserPreference> {
	return {
		locale: navigator.language.includes('zh') ? 'zh-tw' : 'en',
		theme:
			window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
				? 'blue'
				: 'light',
		submit_on_enter: 'false'
	};
}

export const preference = localState<Required<UserPreference>>('preference', defaultPreference());

export function updatePreference(newPreference: Partial<UserPreference>) {
	preference.update((prev) => ({ ...prev, ...newPreference }));
}

async function initWithRemote() {
	const remote = await getRemotePreference();
	if (remote == undefined) return;

	updatePreference(remote.preference);

	await setRemotePreference(get(preference));
}

export async function init() {
	const unsubscribers = [
		preference.subscribe((value) => {
			setTheme(value.theme as any);
			setLocale(value.locale as any);
		}),
		token.subscribe(async (value) => {
			if (value) await initWithRemote();
		})
	];
	onDestroy(() => unsubscribers.forEach((un) => un()));
}

export const submitOnEnter = derived(preference, (x) => x.submit_on_enter);

export const theme = derived(preference, (x) => x.theme);

export const locale = derived(preference, (x) => x.locale);

export const isLightTheme = derived(theme, (x) => isLightThemeFn(x as any));

export const titleGrad = derived(theme, (x) => getTitleGrad(x as any));
