<script lang="ts">
	import { copy } from '$lib/copy';
	import Root from '$lib/markdown/Root.svelte';
	import { CircleDollarSign, FolderSearch, ClipboardCopy } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	let { content = '', token = 0, cost = 0.0 } = $props();
</script>

<div class="group w-full space-y-2 px-10 py-2 wrap-break-word lg:px-20 2xl:px-36">
	<div class="mb-2 border-b border-outline pb-2 select-none">
		<FolderSearch class="mr-2 inline-block" />
		{$_('chat.assistant.response')}
	</div>
	<Root source={content} />
	<div class="flex justify-end space-x-1 opacity-0 group-hover:opacity-100">
		<div class="group/usage relative flex space-x-1">
			<CircleDollarSign
				class="h-10 w-10 rounded-lg p-2 group-hover/usage:bg-hover hover:bg-hover"
			/>

			<div
				class="absolute top-0 right-13 hidden h-10 w-sm items-center justify-end group-hover/usage:flex"
			>
				<div class="rounded-md bg-background p-2 select-none">{token} token/${cost.toFixed(4)}</div>
			</div>
		</div>
		<button onclick={() => copy(content)}>
			<ClipboardCopy class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
		</button>
	</div>
</div>
