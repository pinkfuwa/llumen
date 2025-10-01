<script lang="ts">
	import { FolderSearch } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	let { children } = $props();
</script>

<div class="group w-full space-y-2 px-[5vw] py-2 wrap-break-word lg:px-20 2xl:px-36">
	<div class="mb-2 border-b border-outline pb-2 select-none">
		<FolderSearch class="mr-2 inline-block" />
		{$_('chat.assistant.response')}
	</div>

	{@render children()}
</div>
