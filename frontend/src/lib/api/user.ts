import type { CreateQueryResult, CreateMutationResult } from '@tanstack/svelte-query';
import { createQuery, createMutation } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';

export interface User {
	username: string;
}

async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retrieves a list of users.
 *
 * @returns {Promise<User[]>} A promise that resolves to an array of User objects.
 */
export async function GetUsers(): Promise<User[]> {
	// TODO: Implement user retrieval logic
	await sleep(1000);
	return [{ username: 'user1' }, { username: 'user2' }, { username: 'user3' }];
}

/**
 * Creates a new user with the specified username and password.
 *
 * @param {string} username - The username for the new user.
 * @param {string} password - The password for the new user.
 * @returns {Promise<User>} A promise that resolves to the created User object.
 */
export async function CreateUser(username: string, password: string): Promise<User> {
	// TODO: Implement user creation logic
	await sleep(1000);
	return { username };
}

export function Login(
	username: () => string,
	password: () => string
): CreateMutationResult<{ token: string }, Error, void, unknown> {
	const fetcher = async (username: string, password: string) => {
		console.log('triggered', { username, password });
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
			mutationKey: ['users', state.username, state.password],
			mutationFn: (_: void) => {
				return fetcher(state.username, state.password);
			}
		}))
	);
}
