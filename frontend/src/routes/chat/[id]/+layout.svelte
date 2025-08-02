<script>
	import { goto } from '$app/navigation';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { token } from '$lib/store';

	let { children } = $props();

	$effect(() => {
		if (!$token) goto('/login');
	});
</script>

<div class="flex h-screen flex-row">
	<Sidebar addition />
	<div class="grow">
		{@render children()}
	</div>
</div>
