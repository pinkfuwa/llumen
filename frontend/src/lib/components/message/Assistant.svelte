<script lang="ts">
	import Root from '$lib/markdown/Root.svelte';
	import { CircleDollarSign, FolderSearch } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	let { content = '', token = 1000, cost = 0.01 } = $props();
</script>

<div class="group w-full space-y-2 px-10 py-2 wrap-break-word lg:px-20 2xl:px-36">
	<div class="mb-2 border-b border-outline pb-2 select-none">
		<FolderSearch class="mr-2 inline-block" />
		{$_('chat.assistant.response')}
	</div>
	<Root source={content} />
	<div class="group/usage flex justify-end opacity-0 group-hover:opacity-100">
		<div class="relative">
			<CircleDollarSign
				class="h-10 w-10 rounded-lg p-2 group-hover/usage:bg-hover hover:bg-hover"
			/>
			<div
				class="absolute top-0 right-11 hidden h-10 w-sm items-center justify-end group-hover/usage:flex"
			>
				<div class="rounded-md bg-hover p-2 select-none">{token} token/${cost}</div>
			</div>
		</div>
	</div>
</div>
