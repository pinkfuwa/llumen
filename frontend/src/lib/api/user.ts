import { updatePreference } from '$lib/preference';
import { token } from '$lib/store';
import {
	CreateQuery,
	CreateMockMutation,
	type QueryResult,
	CreateMutation,
	type CreateMutationResult,
	CreateMockQuery
} from './state';
import { APIFetch } from './state/errorHandle';

import type {
	LoginReq,
	LoginResp,
	RenewResp,
	RenewReq,
	UserCreateReq,
	UserCreateResp,
	UserReadResp,
	UserReadReq,
	UserUpdateReq,
	UserUpdateResp
} from './types';

export interface User {
	username: string;
}

export function useUsers(): QueryResult<User[]> {
	return CreateMockQuery([{ username: 'user1' }, { username: 'user2' }, { username: 'user3' }]);
}

export function CreateUser(): CreateMutationResult<UserCreateReq, UserCreateResp> {
	return CreateMutation({
		onSuccess(data, param) {
			// SetQueryData({ key: ['users'], updater: (list) => list });
			// TODO: update user query
		},
		path: 'user/create'
	});
}

export function useUser(): QueryResult<UserReadResp> {
	return CreateQuery<UserReadReq, UserReadResp>({
		key: ['currentUser'],
		path: 'user/read',
		body: {},
		staleTime: 0
	});
}

export function UpdateUser(): CreateMutationResult<UserUpdateReq, UserUpdateResp> {
	return CreateMutation({
		path: 'user/update',
		onSuccess(data, param) {
			if (param.preference) updatePreference(param.preference);
		}
	});
}
