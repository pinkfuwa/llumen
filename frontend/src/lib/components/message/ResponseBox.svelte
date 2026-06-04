<script lang="ts">
	import { FolderSearch } from '@lucide/svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);

	let { children } = $props();
</script>

<div class="group w-full space-y-2 px-[5vw] py-2 wrap-break-word lg:px-20 2xl:px-36">
	<div class="mb-2 border-b border-border pb-2 select-none">
		<FolderSearch class="mr-2 inline-block" />
		{m['chat.assistant.response'](lang)}
	</div>

	{@render children()}
</div>
