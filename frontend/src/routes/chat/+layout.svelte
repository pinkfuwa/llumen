<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	let { children, params } = $props();
	import { Sidebar } from '$lib/components';
	import CollapseBtn from '$lib/components/sidebar/CollapseBtn.svelte';
	import { onDestroy } from 'svelte';

	let addition = $derived(params.id != undefined);
	let open = $state(document.body.clientWidth >= 768);

	function uselessFn(i: any) {}
	$effect(() => {
		uselessFn(page.url);
		if (document.body.clientWidth < 768) open = false;
	});

	function onKeydown(e: KeyboardEvent) {
		if (e.ctrlKey && e.key === 's') {
			e.preventDefault();
			open = !open;
		} else if (e.ctrlKey && e.key === 'd') {
			e.preventDefault();
			open = false;
			goto('/chat/new');
		}
	}

	document.body.addEventListener('keydown', onKeydown);
	onDestroy(() => document.body.removeEventListener('keydown', onKeydown));
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
