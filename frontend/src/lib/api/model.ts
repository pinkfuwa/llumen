import {
	CreateMutation,
	CreateQuery,
	SetQueryData,
	type CreateMutationResult,
	type QueryResult
} from './state';
import type { ModelDeleteReq, ModelDeleteResp, ModelListResp } from './types';

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
		body: {},
		key: ['models']
	});
}

export function DeleteModel(): CreateMutationResult<ModelDeleteReq, ModelDeleteResp> {
	return CreateMutation({
		path: 'user/delete',
		onSuccess(data, param) {
			SetQueryData<ModelListResp>({
				key: ['models'],
				updater: (x) => {
					if (x != undefined) x.list = x.list.filter((u) => u.id !== param.id);
					return x;
				}
			});
		}
	});
}
