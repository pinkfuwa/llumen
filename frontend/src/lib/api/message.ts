import type { CreateQueryResult, CreateMutationResult } from '@tanstack/svelte-query';
import { createQuery, createMutation } from '@tanstack/svelte-query';
import { derived, toStore } from 'svelte/store';

export interface Message {
	chatroomId: string;
	id: string;
	content: string;
	role: 'assistant' | 'user' | 'tool';
	files: Document[];
}

export interface Messages {
	chatroomId: string;
	content: string;
	role: 'assistant' | 'user' | 'tool';
	files: Document[];
}
