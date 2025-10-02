<script lang="ts">
	import { page } from '$app/state';
	let { children, params } = $props();
	import { Sidebar } from '$lib/components';
	import CollapseBtn from '$lib/components/sidebar/CollapseBtn.svelte';

	let addition = $derived(params.id != undefined);
	let open = $state(false);

	function uselessFn(i: any) {}
	$effect(() => {
		uselessFn(page.url);
		if (document.body.clientWidth < 768) open = false;
	});
</script>

<CollapseBtn bind:open />

<div class="relative flex h-screen w-screen flex-row bg-chat-bg">
	<div class="z-20 h-full shrink-0">
		<Sidebar {addition} currentRoom={Number(params.id)} bind:open />
	</div>

	<div class="absolute h-full w-full min-w-0 grow md:static md:w-auto">
		{@render children()}
	</div>
</div>
