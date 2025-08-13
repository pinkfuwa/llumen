import { derived, writable, type Readable, type Writable, readable } from 'svelte/store';
import { sleep } from './api';
import type { Mode } from './model';
import { goto } from '$app/navigation';
import { CreateRecursiveQuery, useMutate } from './state';
import type { mutationResult, RecursiveQueryResult } from './state';

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

// use useRecursiveQuery provide target option, useInfiniteQuery does not
export function useRoom(target: () => HTMLElement | null | undefined, id?: number) {
	const fetcher = async (token: string, id?: number) => {
		console.log('mocking list room(forward)', { token, id });
		await sleep(1000);
		if (token !== '<not-a-token>') throw new Error('Invalid token');
		if (id == undefined) return mockDB.sort((a, b) => b.id - a.id).slice(0, pageSize);
		return mockDB
			.sort((a, b) => b.id - a.id)
			.filter((x) => id == undefined || x.id <= id)
			.slice(0, pageSize);
	};

	return CreateRecursiveQuery<Room[], number | undefined, [number, number | undefined]>({
		initialParam: id,
		nextParam: (list) => {
			if (list.length != pageSize) return;
			return list.at(-1)?.id! - 1;
		},
		genParam: (id, list) => {
			if (list.length == 0) return undefined;
			if (list.length == pageSize) return [list.at(0)!.id, list.at(-1)!.id];
			return [list.at(0)!.id, undefined];
		},
		fetcher: async (param, token) => {
			const id = Array.isArray(param) ? param[0] : param;
			let result = await fetcher(token!, id);
			if (Array.isArray(param) && param[1] != undefined)
				result = result.filter((x) => x.id >= param[1]!);

			return result;
		},
		target,
		key: ['list', 'chatroom']
	}) as RecursiveQueryResult<Room[], number>;
}

const recentRoom = writable<Room[]>([]);

export function useRecentRoom(getId: () => number | undefined): Readable<Room[]> {
	return recentRoom;
}

export interface CreateRoomRequest {
	message: string;
	modelId: string;
	files: File[];
	mode: Mode;
}

export function createRoom(): mutationResult<CreateRoomRequest, Room> {
	const fetcher = async (payload: CreateRoomRequest, token: string): Promise<Room> => {
		console.log('mocking create room', { payload, token });
		if (token !== '<not-a-token>') throw new Error('Invalid token');

		return { id: Date.now(), title: 'New Room', createdAt: Date.now() };
	};

	return useMutate({
		mutator: (param: CreateRoomRequest, token?: string) => fetcher(param, token!),
		onSuccess: (data: Room) => {
			recentRoom.update((x) => [...x, data]);
			goto('/chat/' + encodeURIComponent(data.id));
		}
	});
}
