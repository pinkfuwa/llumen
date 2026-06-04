<script lang="ts">
	import { currentRoom } from '$lib/api';
	let { children } = $props();
	import { Sidebar } from '$lib/components';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
</script>

{#if currentRoom.val && currentRoom.val?.title}
	<title>
		{currentRoom.val.title}
	</title>
{:else}
	<title>
		{m['chat.title'](lang)}
	</title>
{/if}

<div class="bg-chat relative flex h-screen w-screen flex-row">
	<div class="z-20 h-full shrink-0">
		<Sidebar />
	</div>

	<div class="absolute h-full w-full min-w-0 grow md:static md:w-auto">
		{@render children()}
	</div>
</div>
