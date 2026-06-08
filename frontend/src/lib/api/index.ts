export type MutationStatus = 'pending' | 'failed' | 'success' | 'untried';

// Auth
export { Login, RenewToken, type User } from './auth.svelte';

// Chatroom
export {
	chatrooms,
	paginateElement as chatroomElement,
	currentRoom,
	createRoom,
	deleteEntry,
	syncEntry,
	haltCompletion,
	type Entry as Chatroom
} from './chatroom.svelte';

// Files
export {
	upload,
	refresh,
	download,
	downloadCompressed,
	uploadFiles,
	createUploadPipeline
} from './files.svelte';

// Message
export {
	messages,
	streaming,
	paginateElement as messagesElement,
	pushUserMessage,
	createMessage,
	syncMessage,
	deleteMessage
} from './message.svelte';

// Model
export {
	models,
	modelIds,
	deleteModel,
	readModel,
	checkConfig,
	createModel,
	syncModel,
	defaultModelConfig,
	Mode,
	type Capabilty
} from './model.svelte';

// User
export { users, currentUser, createUser, updateUser, deleteUser } from './user.svelte';

// Patch types
export type { PartialAssistantChunk } from './patch';

// Error handling
export { RawAPIFetch, APIFetch, getError, apiBase } from './http.svelte';

// Re-export all generated types
export type * from './types';
