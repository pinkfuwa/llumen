// Re-export all API modules for easier imports and future refactoring

// Auth
export { Login, RenewToken, TryHeaderAuth, initAuth, type User } from './auth';

// Chatroom
export {
	createRoom,
	useRoomsQueryEffect,
	getRoomPages,
	setRoomPages,
	useRoomQueryEffect,
	getCurrentRoom,
	setCurrentRoom,
	haltCompletion,
	deleteRoom,
	updateRoom,
	updateRoomTitle,
	type CreateRoomRequest
} from './chatroom.svelte';

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
	useModelsQueryEffect,
	useModelIdsQueryEffect,
	getModels,
	getModelIds,
	setModels,
	setModelIds,
	deleteModel,
	readModel,
	checkConfig,
	createModel,
	updateModel,
	defaultModelConfig,
	type Capabilty
} from './model.svelte';

// MCP
export {
	useMcpServersQueryEffect,
	getMcpServers,
	createMcpServer,
	deleteMcpServer,
	updateMcpServer,
	readMcpServerConfig,
	checkMcpConfig,
	defaultMcpConfig
} from './mcp.svelte';

// User
export {
	useUsersQueryEffect,
	useUserQueryEffect,
	getUsers,
	getCurrentUser,
	setUsers,
	setCurrentUser,
	createUser,
	updateUser,
	deleteUser,
	type User as UserType
} from './user.svelte';

// Patch types
export type { PartialAssistantChunk } from './patch';

// State management (query/mutation utilities)
export {
	createQueryEffect,
	createMutation,
	createRawMutation,
	createInfiniteQueryEffect,
	insertInfiniteQueryData,
	updateInfiniteQueryDataById,
	removeInfiniteQueryData,
	getInfiniteQueryData,
	CreateMockMutation,
	CreateMockQuery,
	type QueryEffectOption,
	type MutationResult,
	type RawMutationResult,
	type CreateMutationOption,
	type CreateRawMutateOption,
	type InfiniteQueryEffectOption,
	type PageState,
	type Fetcher
} from './state';

// Error handling
export { RawAPIFetch, APIFetch, getError, apiBase } from './state/errorHandle';

// Re-export all generated types
export type * from './types';
