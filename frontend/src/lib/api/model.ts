import { sleep } from './api';
import { useQuery, type QueryResult } from './state/query.svelte';

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

export function useModels(): QueryResult<Model[]> {
	const fetcher = async (token: string): Promise<Model[]> => {
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

	return useQuery({
		param: () => {},
		fetcher: function (_params: void, token?: string): Promise<Model[]> {
			if (!token) throw new Error('Token is required');
			return fetcher(token);
		}
	});
}
