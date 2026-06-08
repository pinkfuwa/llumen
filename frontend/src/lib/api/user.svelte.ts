import { APIFetch } from './http.svelte';
import { token } from '$lib/rune.svelte';

import type {
	UserCreateReq,
	UserCreateResp,
	UserReadResp,
	UserReadReq,
	UserUpdateReq,
	UserUpdateResp,
	UserListResp,
	UserDeleteReq
} from './types';
import type { MutationStatus } from '.';

export const users = $state<{ val?: UserListResp }>({});
export const currentUser = $state<{ val?: UserReadResp }>({});

export async function createUser(req: UserCreateReq): Promise<MutationStatus> {
	const res = await APIFetch<UserCreateResp, UserCreateReq>({
		path: 'user/create',
		body: req,
		token: true
	});
	if (!res) return 'failed';

	if (users.val !== undefined) {
		users.val = {
			...users.val,
			list: [
				{
					id: res.user_id,
					name: req.username
				},
				...users.val.list
			]
		};
	}

	return 'success';
}

export async function updateUser(req: UserUpdateReq): Promise<MutationStatus> {
	const res = await APIFetch<UserUpdateResp, UserUpdateReq>({
		path: 'user/update',
		body: req,
		token: true
	});
	if (!res) return 'failed';
	return 'success';
}

export async function deleteUser(req: UserDeleteReq): Promise<MutationStatus> {
	const res = await APIFetch<UserReadResp, UserDeleteReq>({
		path: 'user/delete',
		body: req,
		token: true
	});
	if (!res) return 'failed';

	if (users.val !== undefined) {
		users.val = {
			...users.val,
			list: users.val.list.filter((u) => u.id !== req.user_id)
		};
	}

	return 'success';
}

$effect.root(() => {
	$effect(() => {
		APIFetch<UserListResp, Record<string, never>>({
			path: 'user/list',
			body: {},
			token: true
		}).then((x) => {
			users.val = x;
		});
	});
});

$effect.root(() => {
	$effect(() => {
		APIFetch<UserReadResp, UserReadReq>({ path: 'user/read', body: {}, token: true }).then((x) => {
			currentUser.val = x;
		});
	});
});
