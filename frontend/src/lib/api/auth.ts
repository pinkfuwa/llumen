import { token } from '$lib/store';
import { page } from '$app/state';
import { goto } from '$app/navigation';
import { CreateMutation, type CreateMutationResult } from './state';
import { APIFetch } from './state/errorHandle';

import type { LoginReq, LoginResp, RenewResp, RenewReq } from './types';
import { onDestroy } from 'svelte';

export interface User {
	username: string;
}

export function Login(): CreateMutationResult<LoginReq, LoginResp> {
	return CreateMutation({
		path: 'auth/login',
		onSuccess: (data) => {
			const now = new Date();
			const expireAt = new Date(data.exp);
			const renewAt = new Date(now.getTime() + (expireAt.getTime() - now.getTime()) / 2);

			token.set({
				value: data.token,
				expireAt: expireAt.toString(),
				renewAt: renewAt.toString()
			});
		}
	});
}

export async function RenewToken(originalToken: string) {
	const res = await APIFetch<RenewResp, RenewReq>('auth/renew', { token: originalToken });

	if (res) {
		const now = new Date();
		const expireAt = new Date(res.exp);
		const renewAt = new Date(now.getTime() + (expireAt.getTime() - now.getTime()) / 2);

		token.set({
			value: res.token,
			expireAt: expireAt.toString(),
			renewAt: renewAt.toString()
		});
	}
}

export function initAuth() {
	const unsubscribers = [
		token.subscribe((token) => {
			const pathname = page.url.pathname;
			console.log('check token and deciding auto route');
			if (token) {
				if (!pathname.startsWith('/chat')) {
					const callback = page.url.searchParams.get('callback');

					if (callback) {
						let url = new URL(decodeURIComponent(callback), document.baseURI);
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
		}),

		token.subscribe((data) => {
			if (data) {
				const expireAt = new Date(data.expireAt);
				const renewAt = new Date(data.renewAt);
				const now = new Date();
				const timeout = renewAt.getTime() - now.getTime();
				if (expireAt < now) {
					token.set(undefined);
				} else if (timeout > 0) {
					const timeoutId = setTimeout(() => RenewToken(data.value), timeout);
					return () => clearTimeout(timeoutId);
				} else {
					RenewToken(data.value);
				}
			}
		})
	];

	onDestroy(() => unsubscribers.forEach((un) => un()));
}
