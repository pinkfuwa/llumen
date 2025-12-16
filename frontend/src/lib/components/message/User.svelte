<script lang="ts">
	import { SquarePen, Check, X } from '@lucide/svelte';
	import FileGroup from '$lib/components/common/FileGroup.svelte';
	import { Markdown } from '$lib/components/markdown';
	import { getStream } from '$lib/api/message.svelte';
	import Button from '$lib/ui/Button.svelte';

	let {
		content = $bindable(''),
		files = [] as Array<{ name: string; id: number }>,
		onupdate = (() => {}) as (text: string, files: Array<{ name: string; id: number }>) => void,
		// streaming means the message just being updated(editing)
		streaming = false
	} = $props();

	// TODO: use component lib
	let editable = $state(false);
	let editBuffer = $state(content);
	let filesBuffer = $state<Array<{ name: string; id: number }>>([]);
	let editFiles = $state<Array<{ name: string; id: number }>>([...files]);

	// bind to markdown height when rendering
	let markdownHeight = $state(0);
	// save height when not rendering
	let renderHeight = $state(0);
	$effect(() => {
		if (editable) return;
		renderHeight = Math.max(24, markdownHeight);
	});

	const actualHeight = $derived.by(() => {
		const contentHeight = (content.split('\n').length + 1) * 24;
		return Math.max(contentHeight, renderHeight);
	});

	let btnGroup = $state<null | HTMLDivElement>(null);
	let scrolled = false;
	$effect(() => {
		if (streaming && !scrolled) {
			btnGroup?.scrollIntoView({
				behavior: 'instant'
			});
			scrolled = true;
		}
	});

	// getStream to bypass svelte bug
	let blockEdit = $state(true);
	getStream((streaming) => (blockEdit = streaming));
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
					{#if editable}
						<FileGroup bind:files={editFiles} deletable={true} />
					{:else}
						<FileGroup files={editFiles} deletable={false} />
					{/if}
				</div>
			{/if}
			{#if editable}
				<div class="flex max-h-[60vh] w-full flex-col-reverse overflow-y-auto">
					<textarea
						class="editor inline field-sizing-content w-full flex-grow resize-none overflow-x-auto"
						bind:value={content}
						style="height: {actualHeight}px"
					></textarea>
				</div>
			{/if}
			<div
				bind:clientHeight={markdownHeight}
				data-state={editable ? 'hide' : 'shown'}
				class="data-[state=hide]:hidden"
			>
				<Markdown source={editBuffer} />
			</div>
		</div>
		<div class="mt-1 flex justify-end" bind:this={btnGroup}>
			<Button
				class="p-2 data-[state=close]:hidden"
				onclick={() => {
					editable = false;
					content = editBuffer;
					editFiles = [...filesBuffer];
				}}
				data-state={editable ? 'open' : 'close'}
				borderless
			>
				<X class="h-6 w-6" />
			</Button>
			<Button
				class="p-2 data-[state=close]:hidden"
				onclick={() => {
					editable = false;
					onupdate(content, editFiles);
				}}
				data-state={editable ? 'open' : 'close'}
				borderless
				disabled={blockEdit}
			>
				<Check class="h-6 w-6" />
			</Button>
			<Button
				class="p-2 group-hover/files:visible data-[state=open]:hidden md:invisible"
				onclick={() => {
					editable = true;
					editBuffer = content;
					filesBuffer = [...editFiles];
				}}
				data-state={editable ? 'open' : 'close'}
				borderless
			>
				<SquarePen class="h-6 w-6" />
			</Button>
		</div>
	</div>
</div>
