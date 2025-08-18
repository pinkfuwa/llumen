import { token } from '$lib/store';
import {
	CreateQuery,
	CreateMockMutation,
	type QueryResult,
	CreateMutation,
	type CreateMutationResult,
	CreateMockQuery
} from './state';
import { apiFetch } from './state/errorHandle';

import type { LoginReq, LoginResp, RenewResp, RenewReq } from './types';

export interface User {
	username: string;
}

export function useUsers(): QueryResult<User[]> {
	return CreateMockQuery([{ username: 'user1' }, { username: 'user2' }, { username: 'user3' }]);
}

interface CreateUserRequest {
	username: string;
	password: string;
}

export function CreateUser(): CreateMutationResult<CreateUserRequest, User> {
	return CreateMockMutation({ username: 'eason' });
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
	const res = await apiFetch<RenewResp, RenewReq>('auth/renew', { token: originalToken });

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
