import { APIFetch } from './http.svelte';
import { token } from '$lib/rune.svelte';

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
import type { MutationStatus } from '.';

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

export const models = $state<{ val?: ModelListResp }>({});
// FIXME: remove modelIds
export const modelIds = $state<{ val?: ModelIdsResp }>({});

export const defaultModelConfig = [
	'display_name="GPT-OSS 20B"',
	'# From https://openrouter.ai/models',
	'# don\'t put "online" suffix.',
	'model_id="openai/gpt-oss-20b"',
	'',
	'# For more settings, see https://pinkfuwa.github.io/llumen/user/config/model'
].join('\n');

export async function deleteModel(req: ModelDeleteReq): Promise<MutationStatus> {
	const res = await APIFetch<ModelDeleteResp, ModelDeleteReq>({
		path: 'model/delete',
		body: req,
		token: true
	});
	if (!res) return 'failed';

	if (models.val !== undefined) {
		models.val = {
			...models.val,
			list: models.val.list.filter((u) => u.id !== req.id)
		};
	}

	return 'success';
}

export async function readModel(id: number): Promise<ModelReadResp> {
	const res = await APIFetch<ModelReadResp, ModelReadReq>({
		path: 'model/read',
		body: { id },
		token: true
	});
	if (res === undefined) throw new Error('No response from server');
	return res;
}

export async function checkConfig(req: ModelCheckReq): Promise<ModelCheckResp | undefined> {
	return APIFetch<ModelCheckResp, ModelCheckReq>({ path: 'model/check', body: req, token: true });
}

export async function createModel(req: ModelCreateReq): Promise<MutationStatus> {
	const res = await APIFetch<ModelCreateResp, ModelCreateReq>({
		path: 'model/create',
		body: req,
		token: true
	});
	if (!res) return 'failed';

	const refreshed = await APIFetch<ModelListResp, Record<string, never>>({
		path: 'model/list',
		body: {},
		token: true
	});
	if (refreshed) models.val = refreshed;

	return 'success';
}

export async function updateModel(req: ModelWriteReq): Promise<MutationStatus> {
	const res = await APIFetch<ModelWriteResp, ModelWriteReq>({
		path: 'model/write',
		body: req,
		token: true
	});
	if (!res) return 'failed';

	if (models.val !== undefined) {
		const updatedList = models.val.list.map((model) =>
			model.id === req.id ? { ...model, display_name: res.display_name } : model
		);
		models.val = {
			...models.val,
			list: updatedList
		};
	}

	return 'success';
}

$effect.root(() => {
	$effect(() => {
		if (!token.value) return;
		APIFetch<ModelListResp, Record<string, never>>({
			path: 'model/list',
			body: {},
			token: true
		}).then((x) => {
			models.val = x;
		});
	});
});

$effect.root(() => {
	$effect(() => {
		if (!token.value) return;
		APIFetch<ModelIdsResp, Record<string, never>>({
			path: 'model/ids',
			body: {},
			token: true
		}).then((x) => {
			modelIds.val = x;
		});
	});
});
