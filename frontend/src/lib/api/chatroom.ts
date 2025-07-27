import type { CreateMutationResult, CreateInfiniteQueryResult } from '@tanstack/svelte-query';
import { useQueryClient } from '@tanstack/svelte-query';
import { createMutation, createInfiniteQuery } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';
import { sleep } from './api';
import { useToken } from '$lib/store';
import type { Mode } from './model';

export interface Rooms {
	list: Room[];
	next: boolean;
}

export interface Room {
	id: string;
	title: string;
}

const pageSize = 12;

export function useRooms(): CreateInfiniteQueryResult<{ pages: Rooms[] }, Error> {
	const token = useToken();

	const fetcher = async (tokenValue: string, page: number) => {
		console.log('mocking list room', { tokenValue, page });
		await sleep(1000);
		if (tokenValue !== '<not-a-token>') throw new Error('Invalid token');
		if (page > 7) return { list: [], next: false };
		const list = Array.from({ length: pageSize }, (_, i) => ({
			id: `${page}-${i}`,
			title: `Room ${page}-${i}`
		}));
		return {
			list,
			next: page < 7
		};
	};

	const state = toStore(() => token.current || '');

	return createInfiniteQuery(
		derived(state, (tokenValue) => ({
			queryKey: ['rooms', tokenValue],
			queryFn: ({ pageParam }: { pageParam: number }) => fetcher(tokenValue, pageParam),
			initialPageParam: 0,
			getNextPageParam: (lastPage: Rooms, allPages: Rooms[]) => {
				console.log(lastPage);
				if (lastPage.next) {
					return allPages.length;
				}
				return undefined;
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

	const queryClient = useQueryClient();

	const fetcher = async (payload: CreateRoomRequest, token: string) => {
		console.log('mocking create room', payload, token);
		if (token !== '<not-a-token>') throw new Error('Invalid token');

		return { id: 'new-chat-room-id', title: 'New Room' };
	};

	return createMutation(
		derived(state, (state) => ({
			mutationFn: (payload: CreateRoomRequest) => fetcher(payload, state)
			// TODO: Optimistic update of infiniteQuery
			//
			// This will override default onSuccess callback
			// Also, setQueryData didn't work for some reason
			//
			// onSuccess: (data: Room, _: CreateRoomRequest) => {
			// 	queryClient.setQueryData(['rooms', state], (old: Rooms) => {
			// 		let list = [data, ...old.list];
			// 		return { list, next: old.next };
			// 	});
			// }
		}))
	);
}
