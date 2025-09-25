<script lang="ts">
	let { children, params } = $props();
	import { Sidebar } from '$lib/components';
	import { page } from '$app/state';

	let addition = $derived(params.id != undefined);
	let collapsed = $state(false);

	$effect(() => {
		if (page.route.id?.endsWith('new')) {
			collapsed = true;
		}
	});
</script>

<div class="relative flex h-screen flex-row bg-chat-bg">
	<div class="absolute z-20 shrink-0 overflow-hidden md:static">
		<Sidebar {addition} currentRoom={Number(params.id)} bind:collapsed />
	</div>
	<div class="absolute h-screen w-full min-w-0 md:static">
		{@render children()}
	</div>
</div>

<!-- <div class="relative bg-chat-bg">
	<div class="absolute z-10 w-full">
		<Sidebar {addition} currentRoom={Number(params.id)} bind:collapsed />
	</div>
	<div class="absolute w-full">
		{@render children()}
	</div>
</div> -->
