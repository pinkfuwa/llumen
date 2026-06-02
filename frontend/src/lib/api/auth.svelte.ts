import { token } from '$lib/store.svelte';
import { page } from '$app/state';
import { goto } from '$app/navigation';
import { createMutation, type MutationResult } from './state';
import { APIFetch } from './state/errorHandle.svelte';

import type { LoginReq, LoginResp, RenewResp, RenewReq, HeaderAuthResp } from './types';

export interface User {
	username: string;
}

function applyToken(data: { token: string; exp: string | number }) {
	const now = new Date();
	const expireAt = new Date(data.exp);
	const renewAt = new Date(now.getTime() + (expireAt.getTime() - now.getTime()) / 2);

	token.value = {
		value: data.token,
		expireAt: expireAt.toString(),
		renewAt: renewAt.toString()
	};
}

export function Login(): MutationResult<LoginReq, LoginResp> {
	return createMutation({
		path: 'auth/login',
		onSuccess: (data) => {
			applyToken(data);
		}
	});
}

export async function RenewToken(originalToken: string) {
	const res = await APIFetch<RenewResp, RenewReq>('auth/renew', { token: originalToken });

	if (res) {
		applyToken(res);
	}
}

export async function TryHeaderAuth() {
	const res = await APIFetch<HeaderAuthResp>('auth/header');

	if (res && res.exp != undefined && res.token) {
		applyToken({ token: res.token, exp: res.exp });
	}
}

$effect.root(() => {
	$effect(() => {
		const pathname = page.url.pathname;
		if (pathname.startsWith('/markdown')) return;
		if (token.value) {
			if (!pathname.startsWith('/chat')) {
				const callback = page.url.searchParams.get('callback');

				if (callback) {
					const url = new URL(decodeURIComponent(callback), document.baseURI);
					if (url.origin == window.location.origin) goto(url);
				} else {
					goto('/chat/new');
				}
			}
		} else if (!pathname.startsWith('/login')) {
			if (pathname.startsWith('/chat') && pathname != '/chat/new')
				goto(`/login?callback=${encodeURIComponent(pathname)}`);
			else goto('/login');
		}
	});

	$effect(() => {
		const data = token.value;
		if (!data) return;
		const expireAt = new Date(data.expireAt);
		const renewAt = new Date(data.renewAt);
		const now = new Date();
		const timeout = renewAt.getTime() - now.getTime();
		if (expireAt < now) {
			token.value = undefined;
		} else if (timeout > 0) {
			const timeoutId = setTimeout(() => RenewToken(data.value), timeout);
			return () => clearTimeout(timeoutId);
		} else {
			RenewToken(data.value);
		}
	});

	return () => {};
});
