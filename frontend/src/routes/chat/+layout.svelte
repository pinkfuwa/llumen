<script>
	import { goto } from '$app/navigation';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { useToken } from '$lib/store';

	let { children } = $props();

	let token = useToken();

	$effect(() => {
		if (token.current == null) goto('/login');
	});
</script>

<div class="flex h-screen flex-row">
	<Sidebar />
	<div class="grow">
		{@render children()}
	</div>
</div>
