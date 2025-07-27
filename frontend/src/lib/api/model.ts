import type { CreateQueryResult } from '@tanstack/svelte-query';
import { createQuery } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';
import { sleep } from './api';
import { useToken } from '$lib/store';

interface Model {
	displayName: string;
	modelId: string;
	capacity: {
		image: boolean;
		audio: boolean;
		document: boolean;
		video: boolean;
	};
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
