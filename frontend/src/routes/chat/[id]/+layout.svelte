<script lang="ts">
	let { children, params } = $props();
	import Sidebar from '$lib/components/Sidebar.svelte';
</script>

<svelte:head>
	<title>Chatroom {params.id}</title>
</svelte:head>
<div class="flex h-screen flex-row">
	<div class="relative w-64 overflow-hidden lg:w-80 xl:w-108">
		<Sidebar addition currentRoom={params.id} />
	</div>
	<div class="nobar relative flex h-full w-full flex-col overflow-y-auto">
		{@render children()}
	</div>
</div>
