import { events } from 'fetch-event-stream';
import { getError, RawAPIFetch } from './errorHandle';
import { onDestroy } from 'svelte';
import { globalCache } from './cache';
import { writable, type Readable } from 'svelte/store';
import { dispatchError } from '$lib/error';

export interface EventQueryOption<D, P> {
	path: string;
	body?: P;
	method?: 'POST' | 'GET' | 'PUT' | 'UPDATE';
	onEvent: (data: D) => void;
	onConnected?: () => void;
	key?: string[];
}

export interface EventQueryResult {
	status: Readable<boolean>;
}

export function CreateEventQuery<D, P = null>(option: EventQueryOption<D, P>): EventQueryResult {
	const { path, body, method, onEvent, onConnected, key } = option;

	const status = key ? globalCache.getOr(key, false) : writable(false);

	const controller = new AbortController();
	let retryTimeout: ReturnType<typeof setTimeout> | null = null;

	const connect = async () => {
		try {
			const res = await RawAPIFetch<P>(path, body, method, controller.signal);
			status.set(true);
			if (onConnected) onConnected();

			const stream = events(res, controller.signal);
			for await (const event of stream) {
				const data = event.data;

				if (data != undefined && data.trim() != ':') {
					const resJson = JSON.parse(data);
					const error = getError(resJson);
					if (error) {
						dispatchError(error.error, error.reason);
					} else {
						onEvent(resJson);
					}
				}
			}

			status.set(false);
			if (retryTimeout) clearTimeout(retryTimeout);
			retryTimeout = setTimeout(connect, 1000);
		} catch (err) {
			status.set(false);
			if (err instanceof DOMException && err.name === 'AbortError') {
				return;
			}
			if (retryTimeout) clearTimeout(retryTimeout);
			retryTimeout = setTimeout(connect, 5000);
		}
	};

	const handleOnline = () => {
		if (retryTimeout) {
			clearTimeout(retryTimeout);
		}
		connect();
	};

	window.addEventListener('online', handleOnline);

	connect();

	onDestroy(() => {
		controller.abort();
		window.removeEventListener('online', handleOnline);
		if (retryTimeout) {
			clearTimeout(retryTimeout);
		}
		status.set(false);
	});

	return { status };
}

export function GetEventQueryStatus(key: string[]): Readable<boolean> {
	return globalCache.getOr(key, false);
}
