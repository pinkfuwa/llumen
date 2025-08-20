<script lang="ts">
	import { FolderSearch } from '@lucide/svelte';
	import type { TokensList } from 'marked';
	import Parser from '$lib/markdown/Parser.svelte';
	import { _ } from 'svelte-i18n';

	const { list }: { list: (TokensList & { monochrome?: boolean })[] } = $props();
</script>

<div class="space-y-2">
	<div class="group w-full space-y-2 px-10 py-2 wrap-break-word lg:px-20 2xl:px-36">
		<div class="mb-2 border-b border-outline pb-2 select-none">
			<FolderSearch class="mr-2 inline-block" />
			{$_('chat.assistant.response')}
		</div>
		{#each list as tokens}
			<Parser {tokens} monochrome={tokens.monochrome} />
		{/each}
		<!-- TODO: add loading status -->
	</div>
</div>
