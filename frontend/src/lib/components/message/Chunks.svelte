<script lang="ts">
	import type { AssistantChunk } from '$lib/api/types';
	import Assistant from './buttons/Assistant.svelte';
	import Reasoning from './buttons/Reasoning.svelte';
	import ResponseError from './buttons/ResponseError.svelte';
	import Result from './buttons/Result.svelte';
	import Tool from './buttons/Tool.svelte';
	import ToolBox from './buttons/ToolBox.svelte';
	import DeepPlan from './buttons/DeepPlan.svelte';
	import DeepStep from './buttons/DeepStep.svelte';
	import DeepReport from './buttons/DeepReport.svelte';

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
		<DeepPlan content={chunk.c} />
	{:else if kind == 'tool_call'}
		{@const toolCall = chunk.c}
		{@const nextChunk = chunks[chunks.indexOf(chunk) + 1]}
		{@const result = nextChunk && nextChunk.t == 'tool_result' && nextChunk.c.id == toolCall.id ? nextChunk.c.response : ''}
		<ToolBox toolname={toolCall.name}>
			<Tool content={toolCall.arg} />
			<Result content={result} />
		</ToolBox>
	{:else if kind == 'error'}
		<ResponseError content={chunk.c} />
	{:else if kind == 'deep_agent'}
		<!-- Handle deep agent if needed -->
	{/if}
{/each}
