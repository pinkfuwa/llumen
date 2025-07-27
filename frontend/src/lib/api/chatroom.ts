import type { CreateMutationResult, CreateInfiniteQueryResult } from '@tanstack/svelte-query';
import { createMutation, createInfiniteQuery } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';
import { sleep } from './api';
import { useRoomPaginateSession, useToken } from '$lib/store';
import type { Mode } from './model';

export interface Room {
	id: number;
	title: string;
	createdAt: number;
}

export function useRooms(): CreateInfiniteQueryResult<{ pages: Room[][] }, Error> {
	const token = useToken();
	const session = useRoomPaginateSession();

	const fetcher = async (token: string, last: { createdAt: number; id: number }) => {
		console.log('mocking list room', { token, last });
		await sleep(1000);
		if (token !== '<not-a-token>') throw new Error('Invalid token');
		const mockDB = Array.from({ length: 40 }, (_, i) => ({
			id: i + 1,
			title: `Room ${i + 1}`,
			createdAt: Date.now() - (i + 1) * 10000
		}));
		const result = mockDB
			.filter((x) => x.createdAt < last.createdAt)
			.slice(0, 12)
			.sort((a, b) => b.createdAt - a.createdAt);
		console.log('result', result);
		return result.sort((a, b) => b.createdAt - a.createdAt);
	};

	const tokenStore = toStore(() => token.current || '');

	return createInfiniteQuery(
		derived([session, tokenStore], ([$session, $token]) => ({
			queryKey: ['rooms', 'paged', $token, $session],
			queryFn: ({ pageParam }: { pageParam: { createdAt: number; id: number } }) =>
				fetcher($token, pageParam),
			initialPageParam: $session,
			getNextPageParam: (lastPage: Room[]) => {
				const last = lastPage.at(-1);
				return last
					? {
							createdAt: last.createdAt,
							id: last.id
						}
					: undefined;
			}
		}))
	);
}

export interface CreateRoomRequest {
	firstMessage: string;
	modelId: string;
	files: File[];
	mode: Mode;
}

export function createRoom(): CreateMutationResult<Room, Error, CreateRoomRequest, unknown> {
	const token = useToken();
	const state = toStore(() => token.current || '');

	const fetcher = async (payload: CreateRoomRequest, token: string): Promise<Room> => {
		console.log('mocking create room', { payload, token });
		if (token !== '<not-a-token>') throw new Error('Invalid token');

		return { id: Date.now(), title: 'New Room', createdAt: Date.now() };
	};

	return createMutation(
		derived(state, (state) => ({
			mutationFn: (payload: CreateRoomRequest) => fetcher(payload, state)
		}))
	);
}
