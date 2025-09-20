<script lang="ts">
	import type { TokensList } from 'marked';
	import { _ } from 'svelte-i18n';
	import ResponseBox from './buttons/ResponseBox.svelte';
	import {
		SseRespEndKind,
		type MessagePaginateRespChunk,
		type SseRespChunkEnd
	} from '$lib/api/types';
	import Chunks from './Chunks.svelte';
	import { addSSEHandler } from '$lib/api/message';
	import AssitantStream from './buttons/AssitantStream.svelte';
	import Reasoning from './buttons/Reasoning.svelte';
	import { MarkdownPatcher, type UIUpdater } from '../markdown/patcher';
	import ToolBox from './buttons/ToolBox.svelte';
	import Tool from './buttons/Tool.svelte';

	let tokens = $state<TokensList[]>([]);
	let reasoning = $state('');

	let toolName = $state('');
	let toolArg = $state('');

	let { chunks = $bindable<MessagePaginateRespChunk[]>([]) } = $props();

	let lastChunkType = $state<'reasoning' | 'assitant'>('reasoning');

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

	addSSEHandler('chunk_end', (data: SseRespChunkEnd) => {
		if (data.kind == SseRespEndKind.Error) {
			// TODO: handle error
		} else if (lastChunkType == 'reasoning') {
			if (reasoning.length != 0)
				chunks.push({
					id: data.id,
					kind: {
						t: 'reasoning',
						c: { context: reasoning }
					}
				});
			reasoning = '';
		} else if (lastChunkType == 'assitant') {
			chunks.push({
				id: data.id,
				kind: {
					t: 'text',
					c: { context: patcher.content }
				}
			});
			patcher.reset();
		}
	});

	addSSEHandler('tool_call', (data) => {
		toolArg = data.args;
		toolName = data.name;
	});
	addSSEHandler('tool_call_end', (data) => {
		chunks.push({
			id: data.chunk_id,
			kind: {
				t: 'tool_call',
				c: {
					name: data.name,
					args: data.args,
					context: data.content
				}
			}
		});
		toolArg = '';
		toolName = '';
	});
	addSSEHandler('reasoning_token', (data) => {
		lastChunkType = 'reasoning';
		reasoning += data.content;
	});
	addSSEHandler('token', (data) => {
		lastChunkType = 'assitant';
		patcher.feed(data.content);
	});
</script>

<ResponseBox>
	<Chunks {chunks} />

	{#if tokens.length != 0}
		<AssitantStream list={tokens} />
	{:else if reasoning.length != 0}
		<Reasoning content={reasoning} />
	{:else if toolName.length != 0}
		<ToolBox toolname={toolName}>
			<Tool content={toolArg} />
		</ToolBox>
	{/if}

	<div class="space-y-4">
		<hr class="mx-3 animate-pulse rounded-md bg-primary p-1" />
		<hr class="mx-3 animate-pulse rounded-md bg-primary p-1" />
	</div>
</ResponseBox>
