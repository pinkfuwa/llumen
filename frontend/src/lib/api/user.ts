import type { CreateQueryResult, CreateMutationResult } from '@tanstack/svelte-query';
import { createQuery, createMutation } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';

export interface User {
	username: string;
}

async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export function GetUsers(token: () => string): CreateQueryResult<{ users: User[] }> {
	const fetcher = async (token: string): Promise<{}> => {
		console.log('mocking get users', { token });
		await sleep(1000);
		if (token != '<not-a-token>') throw new Error('Invalid credentials');
		return { users: [{ username: 'user1' }, { username: 'user2' }, { username: 'user3' }] };
	};

	const state = toStore(() => ({
		token: token()
	}));

	return createQuery(
		derived(state, (state) => ({
			queryKey: ['users'],
			queryFn: () => {
				return fetcher(state.token);
			}
		}))
	);
}

export function CreateUser(
	username: () => string,
	password: () => string,
	token: () => string
): CreateMutationResult<{}, Error, void, unknown> {
	const fetcher = async (username: string, password: string, token: string): Promise<{}> => {
		console.log('mocking create user', { username, password, token });
		await sleep(1000);
		if (token != '<not-a-token>') throw new Error('Invalid credentials');
		throw new Error('User already exists');
	};

	const state = toStore(() => ({
		username: username(),
		password: password(),
		token: token()
	}));

	return createMutation(
		derived(state, (state) => ({
			mutationKey: ['user', 'create', state.username, state.password],
			mutationFn: (_: void) => {
				return fetcher(state.username, state.password, state.token);
			}
		}))
	);
}

export function Login(
	username: () => string,
	password: () => string
): CreateMutationResult<{ token: string }, Error, void, unknown> {
	const fetcher = async (username: string, password: string) => {
		console.log('mocking login', { username, password });
		await sleep(1000);
		if (username === 'admin' && password === 'P@88w0rd') {
			return { token: '<not-a-token>' };
		}
		throw new Error('Invalid credentials');
	};
	const state = toStore(() => ({
		username: username(),
		password: password()
	}));

	return createMutation(
		derived(state, (state) => ({
			mutationKey: ['user', 'login', state.username, state.password],
			mutationFn: (_: void) => {
				return fetcher(state.username, state.password);
			}
		}))
	);
}
