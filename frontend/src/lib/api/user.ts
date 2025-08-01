import { sleep } from './api';
import { useToken } from '$lib/store';
import { useMutate, type mutationResult } from './state/mutate';
import { CreateQuery, type QueryResult } from './state/query.svelte';

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
export function Login(): mutationResult<
	{ username: string; password: string },
	{ value: string; expireAt: number; duration: number }
> {
	const token = useToken();
	const fetcher = async (username: string, password: string) => {
		console.log('mocking login', { username, password });
		await sleep(1000);
		if (username === 'admin' && password === 'P@88w0rd') {
			const expireAt = Date.now() + 24 * 3600 * 7 * 1000;
			return { value: '<not-a-token>', expireAt, duration: 24 * 3600 * 7 };
		}
		throw new Error('Invalid credentials');
	};

	return useMutate({
		mutator: ({ username, password }: LoginRequest, _) => fetcher(username, password),
		onSuccess: (data) => token.set(data)
	});
}

export function HeaderLogin(): mutationResult<
	void,
	{ value: string; expireAt: number; duration: number }
> {
	const token = useToken();
	const fetcher = async () => {
		console.log('mocking header auth');

		throw new Error('Header auth disabled from backend');
	};

	return useMutate({
		mutator: () => fetcher(),
		onSuccess: (data) => token.set(data)
	});
}
