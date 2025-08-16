import { CreateMockQuery, CreateQuery, type QueryResult } from './state';

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
	modelId: number;
	capacity: Capabilty;
}

type ModelListResp = Model[];

export function useModels(): QueryResult<ModelListResp> {
	return CreateMockQuery([
		{
			displayName: 'Gemini-2.5-Flash',
			modelId: 1,
			capacity: {
				image: true,
				audio: true,
				document: true,
				video: true
			}
		},
		{
			displayName: 'Grok-4',
			modelId: 2,
			capacity: {
				image: true,
				audio: true,
				document: true,
				video: false
			}
		}
	]);
}
