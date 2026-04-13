<script lang="ts">
	import type { AssistantChunk } from '$lib/api/types';
	import Assistant from './Assistant.svelte';
	import Citations from './Citations.svelte';
	import Reasoning from './Reasoning.svelte';
	import ResponseError from './ResponseError.svelte';
	import Result from './Result.svelte';
	import Tool from './Tool.svelte';
	import ToolBox from './ToolBox.svelte';
	import DeepResearch from './DeepResearch.svelte';
	import Image from './Image.svelte';
	import Video from './Video.svelte';

	let {
		chunks,
		streaming = false
	}: {
		chunks: AssistantChunk[];
		streaming?: boolean;
	} = $props();
</script>

{#each chunks as chunk}
	{@const kind = chunk.t}
	{#if kind == 'reasoning'}
		<Reasoning content={chunk.c} />
	{:else if kind == 'text'}
		<Assistant content={chunk.c} {streaming} />
	{:else if kind == 'annotation'}
		<!-- annotation was pruned on server-->
	{:else if kind == 'url_citation'}
		<Citations citations={chunk.c} />
	{:else if kind == 'tool_call'}
		{@const toolCall = chunk.c}
		{@const nextChunk = chunks[chunks.indexOf(chunk) + 1]}
		{@const result =
			nextChunk && nextChunk.t == 'tool_result' && nextChunk.c.id == toolCall.id
				? nextChunk.c.response
				: ''}
		{@const resultFiles =
			nextChunk && nextChunk.t == 'tool_result' && nextChunk.c.id == toolCall.id
				? (nextChunk.c as {
						files?: { id: number; name: string; kind?: 'image' | 'video' | 'other' }[];
				  }).files || []
				: []}
		<ToolBox toolname={toolCall.name}>
			<Tool content={toolCall.arg} />
			<Result content={result} />
		</ToolBox>
		{#each resultFiles as file}
			{#if file.kind === 'video'}
				<Video id={file.id} name={file.name} />
			{:else}
				<Image id={file.id} name={file.name} />
			{/if}
		{/each}
	{:else if kind == 'error'}
		<ResponseError content={chunk.c} />
	{:else if kind == 'deep_agent'}
		<DeepResearch plan={chunk.c} {streaming} />
	{:else if kind == 'image'}
		<Image id={chunk.c} />
	{/if}
{/each}
