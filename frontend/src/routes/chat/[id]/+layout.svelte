<script>
	let { children, params } = $props();
	import Sidebar from '$lib/components/Sidebar.svelte';
</script>

<svelte:head>
	<title>Chatroom {params.id}</title>
</svelte:head>
<div class="flex h-screen flex-row">
	<Sidebar addition />
	<div class="flex h-screen grow flex-col justify-center">
		{@render children()}
	</div>
</div>
