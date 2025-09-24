import { dispatchError } from '$lib/error';
import { get } from 'svelte/store';
import type { Error as APIError } from '../types';
import { token } from '$lib/store';
import { dev } from '$app/environment';

export const apiBase = dev ? 'http://localhost:8001/api/' : '/api/';

export function getError(data: any): APIError | undefined {
	if (typeof data === 'object' && data !== null && 'error' in data) {
		return data as APIError;
	}
}

export async function RawAPIFetch<P = any>(
	path: string,
	body: P | null = null,
	method: 'POST' | 'GET' | 'PUT' | 'UPDATE' = 'POST',
	signal?: AbortSignal
): Promise<Response> {
	let tokenVal = get(token)?.value;

	if (path.startsWith('/')) throw new Error('Invalid path');

	const headers: Record<string, string> = {};
	if (!(body instanceof FormData)) headers['Content-Type'] = 'application/json';
	if (tokenVal) headers['Authorization'] = tokenVal;

	const fetchBody = body instanceof FormData ? body : JSON.stringify(body);

	return fetch(apiBase + path, {
		method,
		headers,
		body: fetchBody,
		signal
	});
}

export async function APIFetch<D, P = any>(
	path: string,
	body: P | null = null,
	method: 'POST' | 'GET' | 'PUT' | 'UPDATE' = 'POST'
): Promise<D | undefined> {
	const res = await RawAPIFetch(path, body, method);

	try {
		const resJson: D | APIError = await res.json();
		const error = getError(resJson);
		if (error) dispatchError(error.error, error.reason);
		else return resJson as D;
	} catch (_) {
		dispatchError('API(typeshare)');
	}
}
