<script lang="ts">
	let { children, params } = $props();
</script>

<svelte:head>
	<title>Chatroom {params.id}</title>
</svelte:head>
{@render children()}
