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
	key?: string[];
}

export interface EventQueryResult {
	status: Readable<boolean>;
}

export function CreateEventQuery<D, P = null>(option: EventQueryOption<D, P>): EventQueryResult {
	let { path, body, method, onEvent, key } = option;

	console.log(key);

	const status = key ? globalCache.getOr(key, false) : writable(false);

	const controller = new AbortController();

	onDestroy(() => controller.abort());
	onDestroy(() => status.set(false));

	(async () => {
		const res = await RawAPIFetch<P>(path, body, method, controller.signal);
		console.log('set');
		status.set(true);
		let stream = events(res, controller.signal);
		try {
			for await (let event of stream) {
				const data = event.data;
				console.log('<<', data);

				if (data != undefined && data.trim() != ':') {
					const resJson = JSON.parse(data);
					const error = getError(resJson);
					if (error) dispatchError(error.error, error.reason);
					else onEvent(resJson);
				}
			}
		} catch (err) {
			if (!(err instanceof DOMException && err.name == 'AbortError')) throw err;
		}
	})();

	return { status };
}

export function GetEventQueryStatus(key: string[]) {
	console.log(key);
	return globalCache.getOr(key, false);
}
