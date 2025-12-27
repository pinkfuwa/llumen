// Re-export all API modules for easier imports and future refactoring

// Auth
export { Login, RenewToken, TryHeaderAuth, initAuth, type User } from './auth';

// Chatroom
export {
	createRoom,
	useRooms,
	haltCompletion,
	useRoom,
	deleteRoom,
	updateRoom,
	updateRoomTitle,
	type CreateRoomRequest
} from './chatroom.svelte';

// Document
export type { Document } from './document';

// Files
export {
	upload,
	refresh,
	download,
	downloadCompressed,
	uploadFiles,
	createUploadEffect
} from './files.svelte';

// Message
export {
	useSSEEffect,
	getMessages,
	getStream,
	pushUserMessage,
	createMessage,
	updateMessage
} from './message.svelte';

// Model
export {
	useModels,
	DeleteModel,
	readModel,
	checkConfig,
	createModel,
	updateModel,
	useModelIds,
	defaultModelConfig,
	type Capabilty
} from './model';

// User
export {
	useUsers,
	CreateUser,
	useUser,
	UpdateUser,
	DeleteUser,
	type User as UserType
} from './user';

// Patch types
export type { PartialAssistantChunk } from './patch';

// State management (query/mutation utilities)
export {
	CreateQuery,
	CreateMutation,
	CreateRawMutation,
	CreateInfiniteQuery,
	SetQueryData,
	SetInfiniteQueryData,
	RemoveInfiniteQueryData,
	RevalidateInfiniteQueryData,
	UpdateInfiniteQueryDataById,
	CreateMockMutation,
	CreateMockQuery,
	clearCache,
	type QueryResult,
	type QueryOption,
	type CreateMutationResult,
	type RawMutationResult,
	type CreateRawMutateOption,
	type InfiniteQueryResult,
	type InfiniteQueryOption,
	type Fetcher,
	type PageEntry
} from './state';

// Error handling
export { RawAPIFetch, APIFetch, getError, apiBase } from './state/errorHandle';

// Re-export all generated types
export type * from './types';
