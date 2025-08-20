import { events } from 'fetch-event-stream';
import { RawAPIFetch } from './errorHandle';
import { onDestroy } from 'svelte';

export interface EventQueryOption<D, P> {
	path: string;
	body?: P;
	method?: 'POST' | 'GET' | 'PUT' | 'UPDATE';
	onEvent: (data: D) => void;
}

export function CreateEventQuery<D, P = null>(option: EventQueryOption<D, P>) {
	let { path, body, method, onEvent } = option;

	const controller = new AbortController();

	onDestroy(() => controller.abort());

	(async () => {
		const res = await RawAPIFetch<P>(path, body, method, controller.signal);
		let stream = events(res, controller.signal);
		for await (let event of stream) {
			const data = event.data;
			if (data != undefined) onEvent(JSON.parse(data));
			console.log('<<', data);
		}
	})();
}
