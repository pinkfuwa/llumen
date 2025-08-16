<script lang="ts">
	let { children, params } = $props();
	import Sidebar from '$lib/components/Sidebar.svelte';
	let collapsed = $state(true);
</script>

<svelte:head>
	<title>Chatroom {params.id}</title>
</svelte:head>
<div class="flex h-screen flex-row">
	<div class="shrink-0 overflow-hidden">
		<Sidebar addition currentRoom={Number(params.id)} />
	</div>
	<div class="nobar flex h-full grow flex-col overflow-y-auto">
		{@render children()}
	</div>
</div>
