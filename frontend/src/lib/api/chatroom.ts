import type { CreateMutationResult, CreateInfiniteQueryResult } from '@tanstack/svelte-query';
import { createMutation, createInfiniteQuery } from '@tanstack/svelte-query';
import { derived, writable, type Readable, type Writable } from 'svelte/store';
import { sleep } from './api';
import { useToken } from '$lib/store';
import type { Mode } from './model';
import { goto } from '$app/navigation';

export interface Room {
	id: number;
	title: string;
	createdAt: number;
}

const mockDB = Array.from({ length: 40 }, (_, i) => ({
	id: i + 1,
	title: `Room ${i + 1}`,
	createdAt: Date.now() - (i + 1) * 10000
}));

const roomPaginateSession = writable({ createdAt: Date.now(), id: -1 });
const useRoomPaginateSession = () => roomPaginateSession;
const recentRoom = writable([]);
const useRecentRoom: () => Writable<Room[]> = () => recentRoom;

function useRoomsQuery(): CreateInfiniteQueryResult<{ pages: Room[][] }, Error> {
	const token = useToken();
	const session = useRoomPaginateSession();

	const fetcher = async (token: string, last: { createdAt: number; id: number }) => {
		console.log('mocking list room', { token, last });
		await sleep(1000);
		if (token !== '<not-a-token>') throw new Error('Invalid token');

		const result = mockDB
			.filter((x) => x.createdAt < last.createdAt)
			.slice(0, 12)
			.sort((a, b) => b.createdAt - a.createdAt);
		console.log('result', result);
		return result.sort((a, b) => b.createdAt - a.createdAt);
	};

	return createInfiniteQuery(
		derived([session, token], ([$session, $token]) => ({
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

export function useRooms(): Readable<{
	data: Room[];
	fetchNextPage: () => void;
	hasNextPage: boolean;
	isFetching: boolean;
	isSuccess: boolean;
}> {
	const roomQuery = useRoomsQuery();
	const recentRoom = useRecentRoom();

	return derived([recentRoom, roomQuery], ([$recentRoom, $roomQuery]) => {
		const pages = $roomQuery.data?.pages ?? [];
		const rooms = [...$recentRoom, ...pages.flat()];
		return {
			data: rooms,
			fetchNextPage: () => $roomQuery.fetchNextPage(),
			hasNextPage: $roomQuery.hasNextPage,
			isFetching: $roomQuery.isFetched,
			isSuccess: $roomQuery.isSuccess
		};
	});
}

export interface CreateRoomRequest {
	firstMessage: string;
	modelId: string;
	files: File[];
	mode: Mode;
}

export function createRoom(): CreateMutationResult<Room, Error, CreateRoomRequest, unknown> {
	const token = useToken();
	const recentRoom = useRecentRoom();

	const fetcher = async (payload: CreateRoomRequest, token: string): Promise<Room> => {
		console.log('mocking create room', { payload, token });
		if (token !== '<not-a-token>') throw new Error('Invalid token');

		return { id: Date.now(), title: 'New Room', createdAt: Date.now() };
	};

	return createMutation(
		derived([token, recentRoom], ([$token, $recentRoom]) => ({
			mutationFn: (payload: CreateRoomRequest) => fetcher(payload, $token),
			onSuccess: (data: Room) => {
				recentRoom.set([data, ...$recentRoom]);
				goto('/chat/' + encodeURIComponent(data.id));
			}
		}))
	);
}
