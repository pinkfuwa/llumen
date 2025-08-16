import type { Mode } from './model';
import { goto } from '$app/navigation';
import type { ChatCreateReq, ChatCreateResp, MessageCreateResp, MessageCreateReq } from './types';
import { CreateRawMutation, type RawMutationResult } from './state';
import { apiFetch } from './state/errorHandle';

export interface CreateRoomRequest {
	message: string;
	modelId: number;
	files: File[];
	mode: Mode;
}

export function createRoom(): RawMutationResult<CreateRoomRequest, MessageCreateResp> {
	return CreateRawMutation({
		mutator: async (param) => {
			let chatRes = await apiFetch<ChatCreateResp, ChatCreateReq>('chat/create', {
				model_id: param.modelId
			});
			if (!chatRes) return;

			await goto('/chat/' + encodeURIComponent(chatRes.id));

			return await apiFetch<MessageCreateResp, MessageCreateReq>('message/create', {
				chat_id: chatRes.id,
				text: param.message
			});
		},
		onSuccess: (data) => {
			// TODO: push front the rooms pagination
		}
	});
}
