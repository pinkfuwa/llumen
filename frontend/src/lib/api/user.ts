import { updatePreference } from '$lib/preference';
import {
	CreateQuery,
	type QueryResult,
	CreateMutation,
	type CreateMutationResult,
	SetQueryData,
	CreateMockMutation,
	CreateMockQuery
} from './state';

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

export interface User {
	username: string;
}

export function useUsers(): QueryResult<UserListResp> {
	return CreateQuery({
		path: 'user/list',
		body: {},
		key: ['users']
	});
}

export function CreateUser(): CreateMutationResult<UserCreateReq, UserCreateResp> {
	return CreateMutation({
		onSuccess(data, param) {
			SetQueryData<UserListResp>({
				key: ['users'],
				updater: (list) => {
					if (list != undefined)
						list.list.unshift({
							id: data.user_id,
							name: param.username
						});
					return list;
				}
			});
		},
		path: 'user/create'
	});
}

export function useUser(): QueryResult<UserReadResp> {
	return CreateMockQuery({
		user_id: 1,
		username: 'admin',
		preference: {}
	});
	return CreateQuery<UserReadReq, UserReadResp>({
		key: ['currentUser'],
		path: 'user/read',
		body: {}
	});
}

export function UpdateUser(): CreateMutationResult<UserUpdateReq, UserUpdateResp> {
	return CreateMockMutation(
		{
			user_id: 1
		},
		(param) => {
			if (param.preference) updatePreference(param.preference);
		}
	);
	return CreateMutation({
		path: 'user/update',
		onSuccess(data, param) {
			if (param.preference) updatePreference(param.preference);
		}
	});
}

export function DeleteUser(): CreateMutationResult<UserDeleteReq, UserReadResp> {
	return CreateMutation({
		path: 'user/delete',
		onSuccess(data, param) {
			SetQueryData<UserListResp>({
				key: ['users'],
				updater: (x) => {
					if (x != undefined) x.list = x.list.filter((u) => u.id !== param.user_id);
					return x;
				}
			});
		}
	});
}
