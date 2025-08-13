import { apiBase, sleep } from './api';
import { token } from '$lib/store';
import { useMutate, type mutationResult } from './state/mutate';
import { CreateQuery, type QueryResult } from './state/query.svelte';
import type { LoginReq, LoginResp, RenewResp, RenewReq } from './types';

export interface User {
	username: string;
}

export function useUsers(): QueryResult<User[]> {
	const fetcher = async (token: string): Promise<User[]> => {
		console.log('mocking get users', { token });
		await sleep(1000);
		if (token != '<not-a-token>') throw new Error('Invalid credentials');
		return [{ username: 'user1' }, { username: 'user2' }, { username: 'user3' }];
	};

	return CreateQuery<void, User[]>({
		param: () => {},
		fetcher: function (_: void, token?: string): Promise<User[]> {
			if (!token) throw new Error('Token is required');
			return fetcher(token);
		},
		key: ['users']
	});
}

interface CreateUserRequest {
	username: string;
	password: string;
}

export function CreateUser(): mutationResult<CreateUserRequest, User> {
	const fetcher = async (username: string, password: string, token: string): Promise<User> => {
		console.log('mocking create user', { username, password, token });
		await sleep(1000);
		if (token != '<not-a-token>') throw new Error('Invalid credentials');
		throw new Error('User already exists');
	};

	return useMutate({
		mutator: ({ username, password }: CreateUserRequest, token?: string) => {
			return fetcher(username, password, token!);
		}
	});
}

interface LoginRequest {
	username: string;
	password: string;
}
export function Login(): mutationResult<LoginReq, LoginResp> {
	return useMutate({
		mutator: (body: LoginRequest, _) =>
			fetch(apiBase + 'auth/login', { body: JSON.stringify(body), method: 'POST' }).then(
				(res) => res.json() as Promise<LoginResp>
			),
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
	throw new Error('unable to get expireAt');

	const body: RenewResp = { token: originalToken };

	const response = await fetch(apiBase + 'auth/renew', {
		body: JSON.stringify(body),
		method: 'POST'
	});
	const data = (await response.json()) as RenewResp;
}

export function HeaderLogin(): mutationResult<
	void,
	{ value: string; expireAt: number; duration: number }
> {
	const fetcher = async () => {
		console.log('mocking header auth');

		throw new Error('Header auth disabled from backend');
	};

	return useMutate({
		mutator: () => fetcher(),
		onSuccess: (data) => token.set(data)
	});
}
