import type { CreateMutationResult, CreateInfiniteQueryResult } from '@tanstack/svelte-query';
import { createMutation, createInfiniteQuery } from '@tanstack/svelte-query';
import { derived, get, writable, type Readable, type Writable } from 'svelte/store';
import { sleep } from './api';
import { useToken } from '$lib/store';
import type { Mode } from './model';
import { goto } from '$app/navigation';
import { useSWR } from 'sswr';

export interface Room {
	id: number;
	title: string;
	createdAt: number;
}

export const mockDB = Array.from({ length: 307 }, (_, i) => ({
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
			.slice(0, 30)
			.sort((a, b) => b.createdAt - a.createdAt);
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

const pageSize = 10;

export interface SwrResult<T> {
	data: Writable<T | undefined>;
	revalidate: () => Promise<T>;
	isLoading: Readable<boolean>;
	isValid: Readable<boolean>;
}

export class RoomSession {
	page = 0;
	id?: number;
	isNormalDirection: boolean;
	constructor();
	constructor(id: number, isNormalDirection: boolean);
	constructor(id?: number, isNormalDirection: boolean = true) {
		this.id = id;
		this.isNormalDirection = isNormalDirection;
	}
	next(id: number): RoomSession {
		const session = new RoomSession(id, true);
		session.page = this.page + 1;
		return session;
	}
	previous(id: number): RoomSession {
		const session = new RoomSession(id, false);
		session.page = this.page - 1;
		return session;
	}
}

export function usePagedRoom(session: RoomSession): SwrResult<{
	nextSession?: RoomSession;
	previousSession?: RoomSession;
	list: Room[];
}> {
	const token = useToken();

	class Fetcher {
		idRange?: [number, number];
		session: RoomSession;
		constructor(session: RoomSession) {
			this.session = session;
		}
		get key() {
			return String(this.session.page);
		}
		async fetchHttp(token: string, session: RoomSession) {
			await sleep(1000);
			console.log('mocking list room', { token, session });
			if (token !== '<not-a-token>') throw new Error('Invalid token');
			if (session.isNormalDirection) {
				if (session.id == undefined) return mockDB.sort((a, b) => b.id - a.id).slice(0, pageSize);
				return mockDB
					.sort((a, b) => b.id - a.id)
					.filter((x) => x.id <= (session.id || -Infinity))
					.slice(0, pageSize);
			}
			if (session.id == undefined) throw new Error('reverse paginator must start with id');
			return mockDB
				.sort((a, b) => a.id - b.id)
				.filter((x) => x.id > (session.id || -Infinity))
				.slice(0, pageSize);
		}
		async fetch(token: string) {
			const list = await this.fetchHttp(token, this.session);

			if (this.idRange) {
				const max = Math.max(...this.idRange);
				const min = Math.min(...this.idRange);
				return list.filter((x) => max >= x.id && x.id >= min);
			}
			if (list.length != 0) this.session.id = list[0].id;

			this.idRange = [list[list.length - 1].id, list[0].id];

			return list;
		}
		async fetchPage(token: string): Promise<{
			nextSession?: RoomSession;
			previousSession?: RoomSession;
			list: Room[];
		}> {
			const list = await this.fetch(token);
			let nextSession: RoomSession | undefined;
			let previousSession: RoomSession | undefined;
			if (list.length >= pageSize) {
				if (this.session.isNormalDirection) {
					nextSession = this.session.next(list.at(-1)?.id! - 1);
				} else {
					previousSession = this.session.previous(list.at(0)?.id!);
				}
			}
			console.log({ list, nextSession, previousSession });
			return { list, nextSession, previousSession };
		}
	}

	const fetcher = new Fetcher(session);

	return useSWR(() => JSON.stringify([get(token), fetcher.key]), {
		fetcher: (encoded: string) => fetcher.fetchPage(JSON.parse(encoded)[0])
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
