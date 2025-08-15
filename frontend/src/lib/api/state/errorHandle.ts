import { dispatchError } from '$lib/error';
import { get } from 'svelte/store';
import { apiBase } from '../api';
import type { Error as APIError } from '../types';
import { token } from '$lib/store';

export async function apiFetch<T>(
	path: string,
	body: any = null,
	method: 'POST' | 'GET' | 'PUT' | 'UPDATE' = 'POST'
): Promise<T | undefined> {
	let tokenVal = get(token)?.value;

	if (path.startsWith('/')) throw new Error('Invalid path');

	const headers: Record<string, string> = {};
	headers['Content-Type'] = 'application/json';
	if (tokenVal) headers['Authorization'] = `Bearer ${tokenVal}`;

	const res = await fetch(apiBase + path, {
		method,
		headers,
		body: body ? JSON.stringify(body) : undefined
	});

	const resJson: T | APIError = await res.json();

	if (typeof resJson === 'object' && resJson !== null && 'error' in resJson) {
		dispatchError(resJson.error, resJson.reason);
	} else {
		return resJson as T;
	}
}
