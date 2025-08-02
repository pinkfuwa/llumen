import type { CreateQueryResult, CreateMutationResult } from '@tanstack/svelte-query';
import { createQuery, createMutation } from '@tanstack/svelte-query';
import { derived, readable, toStore } from 'svelte/store';

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

export function createMessageStore(url: string) {
	return readable<Message[]>([], (set) => {
		const eventSource = new EventSource(url);
		const messages: Message[] = [];

		eventSource.onmessage = (event) => {
			try {
				const newMessage: Message = JSON.parse(event.data);
				messages.push(newMessage);
				set([...messages]);
			} catch (error) {
				console.error('Error parsing message:', error);
			}
		};

		eventSource.onerror = () => {
			eventSource.close();
		};

		return () => {
			eventSource.close();
		};
	});
}
