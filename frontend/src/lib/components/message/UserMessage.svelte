<script lang="ts">
	import Root from '$lib/markdown/Root.svelte';
	import { SquarePen, Check } from '@lucide/svelte';
	let { content = $bindable(''), files = [] } = $props();

	let editable = $state(false);
	let width = 0;

	let rows = () => content.split('\n').length;
</script>

<div class="flex w-full justify-end px-10 lg:px-20 xl:px-36">
	<div class="group {editable ? 'w-[75%]' : 'max-w-[75%]'} wrap-break-word">
		<div class="w-full space-y-2 rounded-md bg-background p-4">
			{#if editable}
				<textarea
					class="editor inline field-sizing-content w-full flex-grow resize-none overflow-scroll"
					rows={rows()}
					bind:value={content}
				></textarea>
			{:else}
				<Root source={content} />
			{/if}
		</div>
		<div class="flex justify-end opacity-0 group-hover:opacity-100">
			<button
				onclick={() => {
					editable = !editable;
				}}
			>
				{#if editable}
					<Check class="m-[1px] h-10 w-10 rounded-lg p-2 hover:bg-hover" />
				{:else}
					<SquarePen class="m-[1px] h-10 w-10 rounded-lg p-2 hover:bg-hover" />
				{/if}
			</button>
		</div>
	</div>
</div>
