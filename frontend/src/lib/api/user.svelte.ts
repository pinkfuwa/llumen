import { APIFetch } from './http.svelte';
import { token } from '$lib/rune.svelte';
import { untrack } from 'svelte';

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

export function createUser(req: UserCreateReq): Promise<MutationStatus> {
	const token_ = untrack(() => token.value?.value);
	return APIFetch<UserCreateResp, UserCreateReq>({
		path: 'user/create',
		body: req,
		token: token_
	}).then((res) => {
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
	});
}

export function updateUser(req: UserUpdateReq): Promise<MutationStatus> {
	const token_ = untrack(() => token.value?.value);
	return APIFetch<UserUpdateResp, UserUpdateReq>({
		path: 'user/update',
		body: req,
		token: token_
	}).then((res) => (res ? 'success' : 'failed'));
}

export function deleteUser(req: UserDeleteReq): Promise<MutationStatus> {
	const token_ = untrack(() => token.value?.value);
	return APIFetch<UserReadResp, UserDeleteReq>({
		path: 'user/delete',
		body: req,
		token: token_
	}).then((res) => {
		if (!res) return 'failed';

		if (users.val !== undefined) {
			users.val = {
				...users.val,
				list: users.val.list.filter((u) => u.id !== req.user_id)
			};
		}

		return 'success';
	});
}

$effect.root(() => {
	$effect(() => {
		let stopped = false;

		APIFetch<UserListResp, Record<string, never>>({
			path: 'user/list',
			body: {},
			token: true
		}).then((x) => {
			if (!stopped) users.val = x;
		});

		APIFetch<UserReadResp, UserReadReq>({ path: 'user/read', body: {}, token: true }).then((x) => {
			if (!stopped) currentUser.val = x;
		});

		return () => {
			stopped = true;
		};
	});
});
