import { CreateQuery, type QueryResult } from './state';
import type { ModelListResp } from './types';

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

export function useModels(): QueryResult<ModelListResp> {
	return CreateQuery({
		path: 'model/list',
		body: {}
	});
}
