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
		<ToolBox toolname={toolCall.name}>
			<Tool content={toolCall.arg} />
			<Result content={result} />
		</ToolBox>
	{:else if kind == 'error'}
		<ResponseError content={chunk.c} />
	{:else if kind == 'deep_agent'}
		<DeepResearch plan={chunk.c} {streaming} />
	{:else if kind == 'image'}
		<Image id={chunk.c} />
	{:else if kind == 'mcp_image'}
		<img
			src="data:{chunk.c.mime_type};base64,{chunk.c.data}"
			alt="MCP content"
			class="my-2 max-w-full rounded"
		/>
	{:else if kind == 'mcp_resource'}
		<div class="my-2 rounded border border-outline p-2 text-sm">
			<span class="font-mono text-text-secondary">{chunk.c.uri}</span>
			{#if chunk.c.text}
				<pre class="mt-1 overflow-auto whitespace-pre-wrap">{chunk.c.text}</pre>
			{/if}
		</div>
	{/if}
{/each}
