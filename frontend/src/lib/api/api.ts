import { dev } from '$app/environment';
import {
	derived,
	type Readable,
	type Subscriber,
	type Unsubscriber,
	type readable
} from 'svelte/store';

export const apiBase = dev ? 'http://localhost:8001/' : '/api/';

export async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

// interface QueryObserverResult<T> {
// 	data?: T;
// 	isPending: boolean;
// 	isFetched: boolean;
// 	failCount: number;
// }

// const defaultInterval = 30000;

// class QueryResult<T> implements Readable<QueryObserverResult<T>> {
// 	canFetch: Readable<boolean>;
// 	fetcher: () => Promise<T>;
// 	timeoutId?: number;
// 	failCount = 0;
// 	data?: T;
// 	constructor(canFetch: Readable<boolean>, fetcher: () => Promise<T>) {
// 		this.canFetch = canFetch;
// 		this.fetcher = fetcher;
// 	}
// 	private getResult(isPending: boolean): QueryObserverResult<T> {
// 		return {
// 			data: this.data,
// 			isPending,
// 			isFetched: this.data != undefined,
// 			failCount: this.failCount
// 		};
// 	}
// 	private setInterval(run: Subscriber<QueryObserverResult<T>>) {
// 		if (this.timeoutId) return;

// 	}
// 	private clearInterval() {
// 		if (this.timeoutId) clearInterval(this.timeoutId);
// 		this.timeoutId = undefined;
// 	}
// 	public subscribe(run: Subscriber<QueryObserverResult<T>>): Unsubscriber {
// 		derived(this.canFetch, (canFetch) => {
// 			if (this.timeoutId) {
// 				if (this.canFetch) {
// 				} else {
// 				}
// 			}
// 		});

// 		const getIntervalId = () => this.timeoutId;
// 		return () => {
// 			let intervalId = getIntervalId();
// 			if (intervalId) clearTimeout(intervalId);
// 		};
// 	}
// }
