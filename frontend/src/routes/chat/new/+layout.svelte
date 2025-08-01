<script>
	let { children } = $props();
	import { goto } from '$app/navigation';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { token } from '$lib/store';

	$effect(() => {
		if (!$token) goto('/login');
	});
</script>

<svelte:head>
	<title></title>
</svelte:head>
<div class="flex h-screen flex-row">
	<Sidebar />
	<div class="flex h-screen grow flex-col justify-center">
		{@render children()}
	</div>
</div>
