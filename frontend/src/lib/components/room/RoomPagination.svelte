<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Page from './Page.svelte';
	import { useRoomsQueryEffect, getRoomPages } from '$lib/api/chatroom.svelte';
	import New from './New.svelte';

	let { addition = false } = $props();

	useRoomsQueryEffect();

	let pages = $derived(getRoomPages());
</script>

<ul class="nobar space-y-1 overflow-y-auto text-sm">
	<li
		data-state={addition ? 'show' : 'hide'}
		class="overflow-hidden transition-[max-height] duration-300 ease-out data-[state=hide]:max-h-0 data-[state=show]:max-h-32"
	>
		<New />
	</li>
	{#each pages as page}
		{#key page.no}
			<Page entry={page} />
		{/key}
	{/each}
</ul>
