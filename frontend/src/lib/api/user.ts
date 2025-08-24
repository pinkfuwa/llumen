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

export function Login(): CreateMutationResult<LoginReq, LoginResp> {
	return CreateMutation({
		path: 'auth/login',
		onSuccess: (data) => {
			const now = new Date();
			const expireAt = new Date(data.exp);
			const renewAt = new Date(expireAt.getTime() / 2 + now.getTime());

			token.set({
				value: data.token,
				expireAt: expireAt.toString(),
				renewAt: renewAt.toString()
			});
		}
	});
}

export async function RenewToken(originalToken: string) {
	const res = await APIFetch<RenewResp, RenewReq>('auth/renew', { token: originalToken });

	if (res) {
		const now = new Date();
		const expireAt = new Date(res.exp);
		const renewAt = new Date(expireAt.getTime() / 2 + now.getTime());

		token.set({
			value: res.token,
			expireAt: expireAt.toString(),
			renewAt: renewAt.toString()
		});
	}
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
		path: 'user/update'
	});
}
