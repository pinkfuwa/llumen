import type {
	CreateQueryResult,
	CreateMutationResult,
	CreateInfiniteQueryResult
} from '@tanstack/svelte-query';
import { useQueryClient } from '@tanstack/svelte-query';
import { createQuery, createMutation, createInfiniteQuery } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';
import { sleep } from './api';

export interface Rooms {
	list: Room[];
	next: boolean;
}

export interface Room {
	id: string;
	title: string;
}

const pageSize = 12;

export function listRoom(
	token: () => string
): CreateInfiniteQueryResult<{ pages: Rooms[] }, Error> {
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

	const state = toStore(token);

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
	token: string;
}

export function createRoom(): CreateMutationResult<
	{ title: string },
	Error,
	CreateRoomRequest,
	unknown
> {
	const queryClient = useQueryClient();

	const fetcher = async (payload: CreateRoomRequest) => {
		console.log('mocking create room', payload);
		if (payload.token !== '<not-a-token>') throw new Error('Invalid token');

		return { title: 'New Room' };
	};

	return createMutation({
		mutationFn: (payload: CreateRoomRequest) => fetcher(payload),
		// TODO: Optimistic update of infiniteQuery
		onSuccess: (data, payload) => {
			queryClient.setQueryData(['rooms', payload.token], (old: Rooms) => {
				let list = [{ id: 'new', title: 'New Room' }, ...old.list];
				return { list, next: old.next };
			});
		}
	});
}
