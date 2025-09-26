<script lang="ts">
	import Root from '../../markdown/Root.svelte';
	import { SquarePen, Check, X } from '@lucide/svelte';
	import FileGroup from '../../buttons/FileGroup.svelte';
	import { Button } from 'bits-ui';
	let {
		content = $bindable(''),
		files = $bindable([] as Array<{ name: string; id: number }>),
		onupdate = (() => {}) as (text: string) => void
	} = $props();

	// TODO: use component lib
	let editable = $state(false);
	let editBuffer = $state('');
	let renderHeight = $state(0);

	let actualHeight = $derived(Math.max(48, renderHeight));
</script>

<div class="flex w-full justify-end px-10 lg:px-20 2xl:px-36">
	<div
		class="group/files {editable
			? 'w-full md:w-[75%]'
			: 'max-w-full md:max-w-[75%]'} wrap-break-word"
	>
		<div class="w-full space-y-2 rounded-md bg-user-bg p-4">
			{#if files.length != 0}
				<div class="mb-2 overflow-auto border-b border-outline pb-2">
					<FileGroup bind:files deletable={editable} />
				</div>
			{/if}
			{#if editable}
				<textarea
					class="editor inline field-sizing-content w-full flex-grow resize-none overflow-auto"
					bind:value={content}
					style="height: {actualHeight}px"
				></textarea>
			{:else}
				<div bind:clientHeight={renderHeight}>
					<Root source={content} />
				</div>
			{/if}
		</div>
		<div class="mt-1 flex justify-end group-hover/files:visible md:invisible">
			<Button.Root
				class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover data-[state=close]:hidden"
				onclick={() => {
					editable = false;
					content = editBuffer;
				}}
				data-state={editable ? 'open' : 'close'}
			>
				<X />
			</Button.Root>
			<Button.Root
				class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover data-[state=close]:hidden"
				onclick={() => onupdate(content)}
				data-state={editable ? 'open' : 'close'}
			>
				<Check />
			</Button.Root>
			<Button.Root
				class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover data-[state=open]:hidden"
				onclick={() => {
					editable = true;
					editBuffer = content;
				}}
				data-state={editable ? 'open' : 'close'}
			>
				<SquarePen />
			</Button.Root>
		</div>
	</div>
</div>
