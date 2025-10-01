<script lang="ts">
	import { SquarePen, Check, X } from '@lucide/svelte';
	import FileGroup from '../../buttons/FileGroup.svelte';
	import { Button } from 'bits-ui';
	import { Markdown } from '$lib/components/markdown';
	let {
		content = $bindable(''),
		files = $bindable([] as Array<{ name: string; id: number }>),
		onupdate = (() => {}) as (text: string) => void
	} = $props();

	// TODO: use component lib
	let editable = $state(false);
	let editBuffer = $state(content);
	let renderHeight = $state(0);

	let actualHeight = $state(22);

	$effect(() => {
		if (!editable) actualHeight = Math.max(22, renderHeight);
	});
</script>

<div class="flex w-full justify-end px-[5vw] lg:px-20 2xl:px-36">
	<div
		class="group/files {editable
			? 'w-full md:w-[75%]'
			: 'max-w-full md:max-w-[75%]'} wrap-break-word"
	>
		<div class="w-full rounded-md bg-user-bg p-4">
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
			{/if}
			<div
				bind:clientHeight={renderHeight}
				data-state={editable ? 'hide' : 'shown'}
				class="data-[state=hide]:hidden"
			>
				<Markdown source={editBuffer} />
			</div>
		</div>
		<div class="mt-1 flex justify-end">
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
				class="h-10 w-10 rounded-lg p-2 duration-150 group-hover/files:visible hover:bg-primary hover:text-text-hover data-[state=open]:hidden md:invisible"
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
