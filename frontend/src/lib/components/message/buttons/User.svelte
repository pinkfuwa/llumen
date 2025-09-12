<script lang="ts">
	import Root from '../../markdown/Root.svelte';
	import { SquarePen, Check, X } from '@lucide/svelte';
	import FileGroup from '../../buttons/FileGroup.svelte';
	let { content = $bindable(''), files = $bindable([] as Array<{ name: string }>) } = $props();

	let editable = $state(false);

	let rows = $derived(content.split('\n').length);
</script>

<div class="flex w-full justify-end px-10 lg:px-20 2xl:px-36">
	<div class="group/files {editable ? 'w-[75%]' : 'max-w-[75%]'} wrap-break-word">
		<div class="w-full space-y-2 rounded-md bg-background p-4">
			{#if files.length != 0}
				<div class="mb-2 overflow-scroll border-b border-outline pb-2">
					<FileGroup bind:files deletable={editable} />
				</div>
			{/if}
			{#if editable}
				<textarea
					class="editor inline field-sizing-content w-full flex-grow resize-none overflow-scroll"
					{rows}
					bind:value={content}
				></textarea>
			{:else}
				<Root source={content} />
			{/if}
		</div>
		<div
			class="flex justify-end {editable
				? 'opacity-100'
				: 'opacity-0'} group-hover/files:opacity-100"
		>
			{#if editable}
				<button
					onclick={() => {
						editable = !editable;
					}}
				>
					<X class="m-[1px] h-10 w-10 rounded-lg bg-background p-2 hover:bg-hover" />
				</button>
			{/if}
			<button
				onclick={() => {
					editable = !editable;
				}}
				aria-label="edit user message"
			>
				{#if editable}
					<Check class="m-[1px] h-10 w-10 rounded-lg bg-background p-2 hover:bg-hover" />
				{:else}
					<SquarePen class="m-[1px] h-10 w-10 rounded-lg p-2 hover:bg-hover" />
				{/if}
			</button>
		</div>
	</div>
</div>
