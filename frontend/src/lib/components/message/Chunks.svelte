<script lang="ts">
	import type { PartialMessagePaginateRespChunk } from '$lib/api/patch';
	import Assitant from './buttons/Assitant.svelte';
	import Reasoning from './buttons/Reasoning.svelte';
	import ResponseError from './buttons/ResponseError.svelte';
	import Result from './buttons/Result.svelte';
	import Tool from './buttons/Tool.svelte';
	import ToolBox from './buttons/ToolBox.svelte';

	let {
		chunks,
		monochrome = false
	}: {
		chunks: PartialMessagePaginateRespChunk[];
		monochrome?: boolean;
	} = $props();
</script>

{#each chunks as chunk}
	{@const kind = chunk.kind.t}
	{@const content = 'content' in chunk.kind.c ? chunk.kind.c.content : ''}
	{#if kind == 'reasoning'}
		<Reasoning {content} />
	{:else if kind == 'text'}
		<Assitant {content} {monochrome} />
	{:else if kind == 'tool_call'}
		<ToolBox toolname={chunk.kind.c.name}>
			<Tool content={chunk.kind.c.args} />
			<Result {content} />
		</ToolBox>
	{:else if kind == 'error'}
		<ResponseError {content} />
	{/if}
{/each}
