import { updatePreference } from '$lib/preference';
import { createQueryEffect, createMutation, type MutationResult } from './state';

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

// Module-level state
let users = $state<UserListResp | undefined>(undefined);
let currentUser = $state<UserReadResp | undefined>(undefined);

// Query effects - must be called during component initialization
export function useUsersQueryEffect() {
	createQueryEffect<Record<string, never>, UserListResp>({
		path: 'user/list',
		body: {},
		updateData: (data) => {
			users = data;
		}
	});
}

export function useUserQueryEffect() {
	createQueryEffect<UserReadReq, UserReadResp>({
		path: 'user/read',
		body: {},
		updateData: (data) => {
			currentUser = data;
		}
	});
}

// Getters for reading state
export function getUsers(): UserListResp | undefined {
	return users;
}

export function getCurrentUser(): UserReadResp | undefined {
	return currentUser;
}

// Setters for updating state
export function setUsers(data: UserListResp | undefined) {
	users = data;
}

export function setCurrentUser(data: UserReadResp | undefined) {
	currentUser = data;
}

// Mutations
export function createUser(): MutationResult<UserCreateReq, UserCreateResp> {
	return createMutation({
		onSuccess(data, param) {
			if (users !== undefined) {
				users = {
					...users,
					list: [
						{
							id: data.user_id,
							name: param.username
						},
						...users.list
					]
				};
			}
		},
		path: 'user/create'
	});
}

export function updateUser(): MutationResult<UserUpdateReq, UserUpdateResp> {
	return createMutation({
		path: 'user/update',
		onSuccess(data, param) {
			if (param.preference) updatePreference(param.preference);
		}
	});
}

export function deleteUser(): MutationResult<UserDeleteReq, UserReadResp> {
	return createMutation({
		path: 'user/delete',
		onSuccess(data, param) {
			if (users !== undefined) {
				users = {
					...users,
					list: users.list.filter((u) => u.id !== param.user_id)
				};
			}
		}
	});
}
