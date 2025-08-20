import { dispatchError } from '$lib/error';
import { get } from 'svelte/store';
import type { Error as APIError } from '../types';
import { token } from '$lib/store';
import { dev } from '$app/environment';

export const apiBase = dev ? 'http://localhost:8001/api/' : '/api/';

export async function APIFetch<D, P = any>(
	path: string,
	body: P | null = null,
	method: 'POST' | 'GET' | 'PUT' | 'UPDATE' = 'POST'
): Promise<D | undefined> {
	let tokenVal = get(token)?.value;

	if (path.startsWith('/')) throw new Error('Invalid path');

	const headers: Record<string, string> = {};
	headers['Content-Type'] = 'application/json';
	if (tokenVal) headers['Authorization'] = tokenVal;

	const res = await fetch(apiBase + path, {
		method,
		headers,
		body: body ? JSON.stringify(body) : undefined
	});

	try {
		const resJson: D | APIError = await res.json();
		if (typeof resJson === 'object' && resJson !== null && 'error' in resJson) {
			dispatchError(resJson.error, resJson.reason);
		} else {
			return resJson as D;
		}
	} catch (_) {
		dispatchError('API(typeshare)');
	}
}
