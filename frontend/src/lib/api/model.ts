import {
	CreateMutation,
	CreateQuery,
	SetQueryData,
	type CreateMutationResult,
	type QueryResult
} from './state';
import { APIFetch } from './state/errorHandle';
import type { MutationResult } from './state/mutate';
import type {
	ModelReadReq,
	ModelReadResp,
	ModelDeleteReq,
	ModelDeleteResp,
	ModelListResp,
	ModelCheckResp,
	ModelCheckReq,
	ModelCreateReq,
	ModelCreateResp,
	ModelWriteReq,
	ModelWriteResp
} from './types';

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
		path: 'model/delete',
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

export async function readModel(id: number): Promise<ModelReadResp> {
	const res = await APIFetch<ModelReadResp, ModelReadReq>('model/read', { id });

	if (res == undefined) throw new Error('No response from server');
	return res;
}

export function checkConfig(): MutationResult<ModelCheckReq, ModelCheckResp> {
	return CreateMutation({
		path: 'model/check'
	});
}

export function createModel(): CreateMutationResult<ModelCreateReq, ModelCreateResp> {
	return CreateMutation({
		path: 'model/create',
		onSuccess(data, param) {
			SetQueryData<ModelListResp>({
				key: ['models'],
				updater: (x) => {
					if (x != undefined) x.list = [data, ...x.list];
					return x;
				}
			});
		}
	});
}

export function updateModel(): CreateMutationResult<ModelWriteReq, ModelWriteResp> {
	return CreateMutation({
		path: 'model/write',
		onSuccess(data, param) {
			SetQueryData<ModelListResp>({
				key: ['models'],
				updater: (x) => {
					const model = x?.list.find((u) => u.id === param.id);
					if (model) model.display_name = data.display_name;

					return x;
				}
			});
		}
	});
}

export const defaultModelConfig = [
	'display_name="GPT-OSS 20B"',
	'# From https://openrouter.ai/models',
	'# don\'t put "online" suffix.',
	'openrouter_id="openai/gpt-oss-20b:free"',
	'',
	'[capability]',
	'# allow user to upload image, the model need to support it',
	'# set to false to disallow upload despite its support',
	'image = false',
	'audio = false',
	'# available option: Native, Text, Mistral, Disabled',
	'ocr = "Native"'
].join('\n');
