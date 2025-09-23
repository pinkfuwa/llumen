<script lang="ts">
	import Root from '../../markdown/Root.svelte';
	import { SquarePen, Check, X } from '@lucide/svelte';
	import FileGroup from '../../buttons/FileGroup.svelte';
	import { Button } from 'bits-ui';
	import { dispatchError } from '$lib/error';
	let { content = $bindable(''), files = $bindable([] as Array<{ name: string }>) } = $props();

	let editable = $state(false);

	let rows = $derived(content.split('\n').length);
</script>

<div class="flex w-full justify-end px-10 lg:px-20 2xl:px-36">
	<div class="group/files {editable ? 'w-[75%]' : 'max-w-[75%]'} wrap-break-word">
		<div class="w-full space-y-2 rounded-md bg-user-bg p-4">
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
			class="mt-1 flex justify-end {editable
				? 'opacity-100'
				: 'opacity-0'} group-hover/files:opacity-100"
		>
			{#if editable}
				<Button.Root
					class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover"
					onclick={() => {
						editable = !editable;
					}}
				>
					<X />
				</Button.Root>
				<Button.Root
					class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover"
					onclick={() => {
						dispatchError('internal', 'Disabled in this version');
					}}
					aria-label="edit user message"
				>
					<Check />
				</Button.Root>
			{:else}
				<Button.Root
					class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover"
					onclick={() => {
						editable = !editable;
					}}
					aria-label="edit user message"
				>
					<SquarePen />
				</Button.Root>
			{/if}
		</div>
	</div>
</div>
