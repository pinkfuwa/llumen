import { displayError } from '$lib/error.svelte';
import type { Error as APIError } from './types';
import { token } from '$lib/rune.svelte';
import { dev } from '$app/environment';

export const apiBase = dev
	? (() => {
			const { protocol, hostname } = new URL(window.location.href);
			return `${protocol}//${hostname}:8001/api/`;
		})()
	: '/api/';

export function getError(data: any): APIError | undefined {
	if (typeof data === 'object' && data !== null && 'error' in data) {
		return data as APIError;
	}
}

type HttpMethod = 'POST' | 'GET' | 'PUT' | 'UPDATE';

/**
 * Options for {@link RawAPIFetch} and {@link APIFetch}.
 *
 * @typeParam P - The type of the request body.
 */
export interface RawFetchOptions<P = any> {
	/** API path fragment, e.g. `'message/paginate'`. Must NOT start with `/`. */
	path: string;
	/** Request body. `null` when there is none. */
	body?: P | null;
	/** HTTP method (default `'POST'`). */
	method?: HttpMethod;
	/** Optional `AbortSignal` for cancellation. */
	signal?: AbortSignal;
	/**
	 * Retry on network error (fetch rejection) with exponential backoff.
	 *
	 * Only enable for idempotent queries (read/list/paginate). Never use for
	 * mutations (create/update/delete) or streaming/SSE requests.
	 *
	 * Default `false`. Up to 3 retries with 200ms/400ms/800ms backoff.
	 * Auth and other HTTP-status errors are not retried — only true network
	 * failures (no response at all) trigger a retry.
	 */
	retry?: boolean;
	/**
	 * Token behaviour:
	 * - `false` (default) - no `Authorization` header.
	 * - `true` - reads `token` from the reactive store.
	 * - `undefined` - skip request on non-Raw request.
	 *   In dev mode, throws if not inside `$effect.tracking()`.
	 * - A string - used verbatim as the `Authorization` header value.
	 */
	token?: boolean | string;
}

const MAX_RETRIES = 3;
const RETRY_BASE_DELAY = 200;

function delay(ms: number, signal?: AbortSignal): Promise<void> {
	return new Promise((resolve, reject) => {
		if (signal?.aborted) return reject(new DOMException('Aborted', 'AbortError'));
		const t = setTimeout(() => {
			signal?.removeEventListener('abort', onAbort);
			resolve();
		}, ms);
		const onAbort = () => {
			clearTimeout(t);
			reject(new DOMException('Aborted', 'AbortError'));
		};
		signal?.addEventListener('abort', onAbort, { once: true });
	});
}

/**
 * Low-level fetch wrapper.
 *
 * - Adds `apiBase` prefix, validates path (no leading `/`).
 * - Sets `Content-Type: application/json` unless body is `FormData`.
 * - Respects `opts.token` for the `Authorization` header (see {@link RawFetchOptions.token}).
 *
 * @typeParam P - Request body type (default `any`).
 */
export function RawAPIFetch<P = any>(opts: RawFetchOptions<P>): Promise<Response | undefined> {
	const max_retries = opts.retry ? MAX_RETRIES : 1;

	const path = opts.path;
	const body = opts.body ?? null;
	const method = opts.method ?? 'POST';
	const signal = opts.signal;

	if (path.startsWith('/')) throw new Error('Invalid path');

	const authMode = opts.token;

	const headers: Record<string, string> = {};
	if (!(body instanceof FormData)) headers['Content-Type'] = 'application/json';

	if (authMode === false) {
		// no Authorization header
	} else if (typeof authMode === 'string') {
		headers['Authorization'] = authMode;
	} else {
		if (dev && !$effect.tracking()) {
			throw new Error(
				'RawAPIFetch: token=true requires $effect.tracking(). Pass a string token or use token=false.'
			);
		}
		if (token.value === undefined) return Promise.resolve(undefined);
		const tokenVal = token.value.value;
		if (tokenVal) headers['Authorization'] = tokenVal;
	}

	let fetchBody;
	if (method !== 'GET') {
		if (body instanceof FormData) fetchBody = body;
		else fetchBody = JSON.stringify(body);
	}

	let lastError: unknown;
	return (async () => {
		for (let attempt = 0; attempt <= max_retries; attempt++) {
			if (opts.signal?.aborted) throw new DOMException('Aborted', 'AbortError');
			try {
				return fetch(apiBase + path, {
					method,
					headers,
					body: fetchBody,
					signal
				});
			} catch (err) {
				if (opts.signal?.aborted) throw err;
				lastError = err;
				const isNetworkError = err instanceof TypeError;
				if (!isNetworkError || attempt === max_retries) throw err;
				await delay(RETRY_BASE_DELAY * 2 ** attempt, opts.signal);
			}
		}
	})();
}

/**
 * Typed fetch helper that wraps {@link RawAPIFetch} and parses the JSON response.
 *
 * If the response contains an `{ error, reason }` shape, calls `displayError`
 * and returns `undefined`. Otherwise returns the parsed JSON as `D`.
 * On network/parse errors, displays a generic "maybe backend is disconnected" message
 * (unless the request was aborted).
 *
 * @typeParam D - Response body type.
 * @typeParam P - Request body type (default `any`).
 */
export function APIFetch<D, P = any>(opts: RawFetchOptions<P>): Promise<D | undefined> {
	return RawAPIFetch(opts).then(async (res) => {
		if (res === undefined) return;
		try {
			const resJson: D | APIError = await res.json();
			const error = getError(resJson);
			if (error) displayError(error.error, error.reason);
			else return resJson as D;
		} catch (_) {
			if (res.status == 429) displayError('API(typeshare)', 'rate limit exceeded');
			else if (!opts.signal?.aborted)
				displayError('API(typeshare)', 'maybe backend is disconnected');
		}
	});
}
