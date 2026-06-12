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
	ModelIdsResp,
	ModelList
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

// user created model
export const models = $state<{ val?: ModelList[] }>({});
// list of all possible model id
export const modelIds = $state<{ val?: string[] }>({});

export const defaultModelConfig = [
	'display_name="GPT-OSS 20B"',
	'# From https://openrouter.ai/models',
	'# don\'t put "online" suffix.',
	'model_id="openai/gpt-oss-20b"',
	'',
	'# For more settings, see https://pinkfuwa.github.io/llumen/user/config/model'
].join('\n');

export async function readModel(id: number): Promise<ModelReadResp> {
	return APIFetch<ModelReadResp, ModelReadReq>({
		path: 'model/read',
		body: { id },
		token: true
	}).then((x) => x ?? { raw: '' });
}

async function fetchModel(): Promise<void> {
	APIFetch<ModelListResp, Record<string, never>>({
		path: 'model/list',
		body: {},
		token: true
	}).then((x) => {
		models.val = x?.list;
	});
}

export async function deleteModel(req: ModelDeleteReq): Promise<MutationStatus> {
	const res = await APIFetch<ModelDeleteResp, ModelDeleteReq>({
		path: 'model/delete',
		body: req,
		token: token.value?.value
	});
	if (!res || !res.deleted) return 'failed';

	let modelIdx = models.val?.findIndex((m) => m.id == req.id);
	console.log('to delete', modelIdx, models);
	if (modelIdx !== undefined && models.val !== undefined) {
		models.val.splice(modelIdx, 1);
	}

	return 'success';
}

export async function checkConfig(req: ModelCheckReq): Promise<ModelCheckResp | undefined> {
	return APIFetch<ModelCheckResp, ModelCheckReq>({ path: 'model/check', body: req, token: true });
}

export async function createModel(req: ModelCreateReq): Promise<MutationStatus> {
	const res = await APIFetch<ModelCreateResp, ModelCreateReq>({
		path: 'model/create',
		body: req,
		token: token.value?.value
	});
	if (!res) return 'failed';

	models.val?.push(res);

	return 'success';
}

export async function syncModel(req: ModelWriteReq): Promise<MutationStatus> {
	const res = await APIFetch<ModelWriteResp, ModelWriteReq>({
		path: 'model/write',
		body: req,
		token: token.value?.value
	});
	if (!res) return 'failed';

	fetchModel();

	return 'success';
}

$effect.root(() => {
	$effect(() => {
		let stopped = false;

		APIFetch<ModelListResp, Record<string, never>>({
			path: 'model/list',
			body: {},
			token: true
		}).then((x) => {
			if (!stopped) models.val = x?.list;
		});

		APIFetch<ModelIdsResp, Record<string, never>>({
			path: 'model/ids',
			body: {},
			token: true
		}).then((x) => {
			if (!stopped) modelIds.val = x?.ids;
		});

		return () => {
			stopped = true;
		};
	});
});
