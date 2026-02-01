import { dev } from '$app/environment';
import { createMutation, createQueryEffect, type MutationResult } from './state';
import { APIFetch } from './state/errorHandle';
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
	ModelWriteResp,
	ModelIdsResp
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

// Module-level state
let models = $state<ModelListResp | undefined>(undefined);
let modelIds = $state<ModelIdsResp | undefined>(undefined);

// Query effects - must be called during component initialization
export function useModelsQueryEffect() {
	if (dev) $inspect('models', models);
	createQueryEffect<Record<string, never>, ModelListResp>({
		path: 'model/list',
		body: {},
		updateData: (data) => {
			models = data;
		}
	});
}

export function useModelIdsQueryEffect() {
	if (dev) $inspect('modelIds', modelIds);
	createQueryEffect<Record<string, never>, ModelIdsResp>({
		path: 'model/ids',
		body: {},
		updateData: (data) => {
			modelIds = data;
		}
	});
}

// Getters for reading state
export function getModels(): ModelListResp | undefined {
	return models;
}

export function getModelIds(): ModelIdsResp | undefined {
	return modelIds;
}

// Setters for updating state
export function setModels(data: ModelListResp | undefined) {
	models = data;
}

export function setModelIds(data: ModelIdsResp | undefined) {
	modelIds = data;
}

// Mutations
export function deleteModel(): MutationResult<ModelDeleteReq, ModelDeleteResp> {
	return createMutation({
		path: 'model/delete',
		onSuccess(data, param) {
			if (models !== undefined) {
				models = {
					...models,
					list: models.list.filter((u) => u.id !== param.id)
				};
			}
		}
	});
}

export async function readModel(id: number): Promise<ModelReadResp> {
	const res = await APIFetch<ModelReadResp, ModelReadReq>('model/read', { id });

	if (res === undefined) throw new Error('No response from server');
	return res;
}

export function checkConfig(): MutationResult<ModelCheckReq, ModelCheckResp> {
	return createMutation({
		path: 'model/check'
	});
}

export function createModel(): MutationResult<ModelCreateReq, ModelCreateResp> {
	return createMutation({
		path: 'model/create',
		onSuccess(data, param) {
			if (models !== undefined) {
				models = {
					...models,
					list: [...models.list, data]
				};
			}
		}
	});
}

export function updateModel(): MutationResult<ModelWriteReq, ModelWriteResp> {
	return createMutation({
		path: 'model/write',
		onSuccess(data, param) {
			if (models !== undefined) {
				const updatedList = models.list.map((model) =>
					model.id === param.id ? { ...model, display_name: data.display_name } : model
				);
				models = {
					...models,
					list: updatedList
				};
			}
		}
	});
}

export const defaultModelConfig = [
	'display_name="GPT-OSS 20B"',
	'# From https://openrouter.ai/models',
	'# don\'t put "online" suffix.',
	'model_id="openai/gpt-oss-20b"',
	'',
	'# For more settings, see https://llumen-docs.easonabc.eu.org/user-guide/model-config'
].join('\n');
