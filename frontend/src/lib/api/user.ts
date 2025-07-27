import type { CreateQueryResult, CreateMutationResult } from '@tanstack/svelte-query';
import { createQuery, createMutation } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';
import { sleep } from './api';
import { useToken } from '$lib/store';

export interface User {
	username: string;
}

export function useUsers(): CreateQueryResult<{ users: User[] }> {
	const token = useToken();

	const fetcher = async (token: string): Promise<{}> => {
		console.log('mocking get users', { token });
		await sleep(1000);
		if (token != '<not-a-token>') throw new Error('Invalid credentials');
		return { users: [{ username: 'user1' }, { username: 'user2' }, { username: 'user3' }] };
	};

	return createQuery(
		derived(token, (token) => ({
			queryKey: ['users'],
			queryFn: () => {
				return fetcher(token);
			}
		}))
	);
}

interface CreateUserRequest {
	username: string;
	password: string;
	token: string;
}

export function CreateUser(): CreateMutationResult<{}, Error, CreateUserRequest, unknown> {
	const fetcher = async (username: string, password: string, token: string): Promise<{}> => {
		console.log('mocking create user', { username, password, token });
		await sleep(1000);
		if (token != '<not-a-token>') throw new Error('Invalid credentials');
		throw new Error('User already exists');
	};

	return createMutation({
		mutationFn: ({ username, password, token }: CreateUserRequest) => {
			return fetcher(username, password, token);
		}
	});
}

interface LoginRequest {
	username: string;
	password: string;
}
export function Login(): CreateMutationResult<{ token: string }, Error, LoginRequest, unknown> {
	const fetcher = async (username: string, password: string) => {
		console.log('mocking login', { username, password });
		await sleep(1000);
		if (username === 'admin' && password === 'P@88w0rd') {
			return { token: '<not-a-token>' };
		}
		throw new Error('Invalid credentials');
	};

	return createMutation({
		mutationFn: ({ username, password }: LoginRequest) => {
			return fetcher(username, password);
		}
	});
}
