import type { CreateQueryResult } from '@tanstack/svelte-query';
import { createQuery } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';
import { sleep } from './api';
import { useToken } from '$lib/store';

export enum Mode {
	DEEP = 2,
	SEARCH = 1,
	NORMAL = 0
}

export interface Capabilty {
	image: boolean;
	audio: boolean;
	document: boolean;
	video: boolean;
}
interface Model {
	displayName: string;
	modelId: string;
	capacity: Capabilty;
}

export function useModels(): CreateQueryResult<Model[]> {
	const token = useToken();

	const fetcher = async (token: string) => {
		console.log('mocking list models', { token });
		await sleep(1000);
		return [
			{
				displayName: 'Gemini-2.5-Flash',
				modelId: 'google/gemini-2.5-flash',
				capacity: {
					image: true,
					audio: true,
					document: true,
					video: true
				}
			},
			{
				displayName: 'Grok-4',
				modelId: 'xai/grok-4-beta',
				capacity: {
					image: true,
					audio: true,
					document: true,
					video: false
				}
			}
		];
	};
	let state = toStore(() => token.current || '');

	return createQuery(
		derived(state, (stateValue) => ({
			queryKey: ['models', stateValue],
			queryFn: async () => fetcher(stateValue)
		}))
	);
}
