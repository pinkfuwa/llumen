<script lang="ts">
	import { SquarePen, Check, X, Trash2 } from '@lucide/svelte';
	import FileGroup from '$lib/components/common/FileGroup.svelte';
	import { Markdown } from '$lib/components/markdown';
	import { syncMessage, deleteMessage, streaming, type MutationStatus } from '$lib/api';
	import { shouldSubmitOnEnter } from '$lib/components/input/submitOnEnter.svelte';
	import Button from '$lib/ui/Button.svelte';

	let {
		content = $bindable(''),
		files = [] as Array<{ name: string; id: number }>,
		id: id
	}: {
		content: string;
		files?: Array<{ name: string; id: number }>;
		id: number;
	} = $props();

	let submissionStatus = $state<MutationStatus>('untried');
	let isEditing = $state(false);
	let editBuffer = $state(content);
	let filesBuffer = $state<Array<{ name: string; id: number }>>([]);
	let editFiles = $state<Array<{ name: string; id: number }>>([]);

	let disabled = $derived(streaming.val || submissionStatus == 'pending');

	$effect(() => {
		if (isEditing) return;
		editBuffer = content;
		editFiles = [...files];
		filesBuffer = [...files];
	});

	const actualHeight = $derived.by(() => {
		const contentHeight = (content.split('\n').length + 1) * 24;
		return Math.max(contentHeight, renderHeight);
	});

	// bind to markdown height when rendering
	let markdownHeight = $state(0);
	// save height when not rendering
	let renderHeight = $state(0);

	$effect(() => {
		if (isEditing) return;
		renderHeight = Math.max(24, markdownHeight);
	});

	let virtualKeyboard = $state(false);
	if ('virtualKeyboard' in navigator) {
		navigator.virtualKeyboard.overlaysContent = true;

		navigator.virtualKeyboard.addEventListener('geometrychange', () => {
			virtualKeyboard = true;
			navigator.virtualKeyboard.overlaysContent = false;
		});
	}

	function submitEdit() {
		isEditing = false;
		submissionStatus = 'pending';
		syncMessage(id, content, editFiles).then((x) => {
			submissionStatus = x;
		});
	}
</script>

<div class="group/files mt-4 w-full px-[5vw] lg:px-20 2xl:px-36" id={`${id}`}>
	<div class="flex justify-end">
		<div
			class="rounded-md bg-muted p-4 wrap-break-word data-[state=edit]:w-full data-[state=text]:max-w-full data-[state=edit]:md:w-[calc(100%-2rem)] data-[state=text]:md:max-w-[calc(100%-2rem)]"
			data-state={isEditing ? 'edit' : 'text'}
		>
			{#if files.length != 0}
				{@const separator = isEditing || content.trim().length > 0}
				<div class="mb-2 overflow-auto{separator ? ' border-b border-border pb-2' : ''}">
					{#if isEditing}
						<FileGroup bind:files={editFiles} deletable={true} />
					{:else}
						<FileGroup files={editFiles} deletable={false} />
					{/if}
				</div>
			{/if}
			{#if isEditing}
				<div class="flex max-h-[60vh] w-full flex-col-reverse overflow-y-auto">
					<textarea
						class="editor inline field-sizing-content w-full flex-grow resize-none overflow-x-auto"
						bind:value={content}
						style="height: {actualHeight}px"
						onkeypress={(event) => {
							if (shouldSubmitOnEnter(event, { virtualKeyboard })) {
								event.preventDefault();
								submitEdit();
							}
						}}
					></textarea>
				</div>
			{/if}
			<div
				bind:clientHeight={markdownHeight}
				data-state={isEditing ? 'hide' : 'shown'}
				class="data-[state=hide]:hidden"
			>
				<Markdown source={editBuffer} />
			</div>
		</div>
	</div>
	<div class="mt-1 flex justify-end">
		<Button
			class="p-2 data-[state=close]:hidden"
			onclick={() => {
				isEditing = false;
				content = editBuffer;
				editFiles = [...filesBuffer];
			}}
			data-state={isEditing ? 'open' : 'close'}
			borderless
		>
			<X class="h-6 w-6" />
		</Button>
		<Button
			class="p-2 data-[state=close]:hidden"
			onclick={submitEdit}
			data-state={isEditing ? 'open' : 'close'}
			borderless
			{disabled}
		>
			<Check class="h-6 w-6" />
		</Button>
		<Button
			class="p-2 group-hover/files:visible data-[state=open]:hidden md:invisible"
			onclick={() => {
				isEditing = true;
				editBuffer = content;
				filesBuffer = [...editFiles];
			}}
			data-state={isEditing ? 'open' : 'close'}
			borderless
		>
			<SquarePen class="h-6 w-6" />
		</Button>
		<Button
			class="p-2 group-hover/files:visible data-[state=open]:hidden md:invisible"
			onclick={() => {
				deleteMessage(id);
			}}
			data-state={isEditing ? 'open' : 'close'}
			borderless
			{disabled}
		>
			<Trash2 class="h-6 w-6" />
		</Button>
	</div>
</div>
