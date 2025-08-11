<script>
	let { children } = $props();
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { _ } from 'svelte-i18n';
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>

<div class="flex h-screen flex-row">
	<div class="shrink-0 overflow-hidden">
		<Sidebar />
	</div>
	<div class="flex h-full w-full grow flex-col justify-center">
		{@render children()}
	</div>
</div>
