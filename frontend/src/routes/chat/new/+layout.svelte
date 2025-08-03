<script>
	let { children } = $props();
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { _ } from 'svelte-i18n';
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>

<div class="flex h-screen flex-row">
	<Sidebar />
	<div class="flex h-screen grow flex-col justify-center">
		{@render children()}
	</div>
</div>
