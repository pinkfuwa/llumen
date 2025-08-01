<script>
	let { children } = $props();
	import { goto } from '$app/navigation';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { useToken } from '$lib/store';

	let token = useToken();

	$effect(() => {
		if ($token == '') goto('/login');
		// if (token.current == '') goto('/login');
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
