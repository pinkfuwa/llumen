import { APIFetch } from './errorHandle.svelte';
import { token } from '$lib/store.svelte';

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

$effect.root(() => {
	if (!token.value) return;
	APIFetch<UserListResp, Record<string, never>>('user/list', {}).then((x) => {
		users.val = x;
	});
});

$effect.root(() => {
	if (!token.value) return;
	APIFetch<UserReadResp, UserReadReq>('user/read', {}).then((x) => {
		currentUser.val = x;
	});
});

export async function createUser(req: UserCreateReq): Promise<MutationStatus> {
	const res = await APIFetch<UserCreateResp, UserCreateReq>('user/create', req);
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
	const res = await APIFetch<UserUpdateResp, UserUpdateReq>('user/update', req);
	if (!res) return 'failed';
	return 'success';
}

export async function deleteUser(req: UserDeleteReq): Promise<MutationStatus> {
	const res = await APIFetch<UserReadResp, UserDeleteReq>('user/delete', req);
	if (!res) return 'failed';

	if (users.val !== undefined) {
		users.val = {
			...users.val,
			list: users.val.list.filter((u) => u.id !== req.user_id)
		};
	}

	return 'success';
}
