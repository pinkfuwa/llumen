<script lang="ts">
	import { _ } from 'svelte-i18n';
	import MessageInput from '$lib/components/MessageInput.svelte';
	import Root from '$lib/markdown/Root.svelte';
	import MockMessage from '$lib/mockMessages.json';
	import { FolderSearch } from '@lucide/svelte';
</script>

<div class="w-full grow space-y-2 pb-4">
	{#each MockMessage as msg, i}
		{#if i % 2 == 0}
			<div class="flex w-full justify-end px-10 lg:px-20 xl:px-36">
				<div class="max-w-[75%] space-y-2 rounded-md bg-background p-4 wrap-break-word">
					<Root source={msg} />
				</div>
			</div>
		{:else}
			<div class="w-full space-y-2 px-10 py-4 wrap-break-word lg:px-20 xl:px-36">
				<div class="mb-2 border-b border-outline pb-2">
					<FolderSearch class="mr-2 inline-block" />
					Answer
				</div>
				<Root source={msg} />
			</div>
		{/if}
	{/each}
</div>
<div class="sticky bottom-2 mt-auto flex justify-center">
	<MessageInput above />
</div>
