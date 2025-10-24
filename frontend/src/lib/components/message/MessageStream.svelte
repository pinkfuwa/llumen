<script lang="ts">
	import type { TokensList } from 'marked';
	import { _ } from 'svelte-i18n';
	import ResponseBox from './buttons/ResponseBox.svelte';
	import Chunks from './Chunks.svelte';
	import { addSSEHandler } from '$lib/api/message';
	import AssitantStream from './buttons/AssitantStream.svelte';
	import Reasoning from './buttons/Reasoning.svelte';
	import { MarkdownPatcher, type UIUpdater } from '../markdown/patcher';
	import ToolBox from './buttons/ToolBox.svelte';
	import Tool from './buttons/Tool.svelte';
	import type { PartialMessagePaginateRespChunk } from '$lib/api/patch';
	import { MessagePaginateRespRole, type MessagePaginateRespList } from '$lib/api/types';
	import { SetInfiniteQueryData } from '$lib/api/state';
	import { useRoomStreamingState } from '$lib/api/chatroom';
	import { heatMarkdownCache } from '../markdown';

	let { chat_id } = $props<{ chat_id: number }>();

	let chunks = $state<PartialMessagePaginateRespChunk[]>([]);

	let tokens = $state<TokensList[]>([]);
	let reasoning = $state('');

	let toolName = $state('');
	let toolArg = $state('');

	let isStreaming = $derived(useRoomStreamingState(chat_id));

	let lastChunkType = $state<'reasoning' | 'token' | null>(null);

	const updater: UIUpdater = {
		reset() {
			tokens = [];
		},
		append(newTokens) {
			tokens.push(newTokens);
		},
		replace(newTokens) {
			tokens.pop();
			tokens.push(newTokens);
		}
	};

	const patcher = new MarkdownPatcher(updater);

	addSSEHandler('reasoning', (data) => {
		isStreaming.set(true);
		reasoning += data.content;
		if (lastChunkType == 'token') {
			chunks.push({
				kind: {
					t: 'text',
					c: { content: patcher.content }
				}
			});
			patcher.reset();
		}
		lastChunkType = 'reasoning';
	});
	addSSEHandler('token', (data) => {
		isStreaming.set(true);
		patcher.feed(data.content);
		if (lastChunkType == 'reasoning') {
			chunks.push({
				kind: {
					t: 'reasoning',
					c: { content: reasoning }
				}
			});
			reasoning = '';
		}
		lastChunkType = 'token';
	});

	addSSEHandler('tool_call', (data) => {
		toolArg = data.args;
		toolName = data.name;
	});
	addSSEHandler('tool_result', (data) => {
		chunks.push({
			kind: {
				t: 'tool_call',
				c: {
					name: toolName,
					args: toolArg,
					content: data.content
				}
			}
		});
		toolArg = '';
		toolName = '';
	});

	addSSEHandler('error', (data) => {
		chunks.push({
			kind: {
				t: 'error',
				c: {
					content: data.content
				}
			}
		});
	});

	addSSEHandler('complete', async (data) => {
		if (lastChunkType == 'reasoning') {
			chunks.push({
				kind: {
					t: 'reasoning',
					c: { content: reasoning }
				}
			});
		}
		if (lastChunkType == 'token') {
			chunks.push({
				kind: {
					t: 'text',
					c: { content: patcher.content }
				}
			});
		}

		let chunk_ids = data.chunk_ids.toReversed();

		await Promise.all(
			chunks
				.filter((c) => c.kind.t == 'text')
				.map((c) => heatMarkdownCache((c.kind.c as any).content))
		);

		SetInfiniteQueryData<MessagePaginateRespList>({
			key: ['messagePaginate', chat_id.toString()],
			data: {
				id: data.id,
				role: MessagePaginateRespRole.Assistant,
				chunks: chunks.map((x) => ({ id: chunk_ids.pop()!, ...x })),
				price: data.cost,
				token: data.token_count
			}
		});
		isStreaming.set(false);
		chunks = [];
		reasoning = '';
		patcher.reset();
	});

	// TODO: revalidate on version change
	// addSSEHandler('version', (data) => { });

	addSSEHandler('start', () => {
		isStreaming.set(true);
	});

	addSSEHandler('connect', () => {
		isStreaming.set(false);
		chunks = [];
		reasoning = '';
		patcher.reset();
	});
</script>

{#if $isStreaming}
	<ResponseBox>
		<Chunks {chunks} />

		{#if tokens.length != 0}
			<AssitantStream list={tokens} />
		{:else if reasoning.length != 0}
			<Reasoning content={reasoning} open />
		{:else if toolName.length != 0}
			<ToolBox toolname={toolName}>
				<Tool content={toolArg} />
			</ToolBox>
		{/if}

		<div class="space-y-4">
			<hr class="mx-3 animate-pulse rounded-md border-primary bg-primary p-1" />
			<hr class="mx-3 animate-pulse rounded-md border-primary bg-primary p-1" />
		</div>
	</ResponseBox>
{/if}
