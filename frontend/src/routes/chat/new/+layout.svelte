<script>
	let { children } = $props();
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { _ } from 'svelte-i18n';
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>

<div class="flex h-screen flex-row">
	<div class="relative w-64 overflow-hidden lg:w-80 xl:w-108">
		<Sidebar />
	</div>
	<div class="relative flex h-full w-full flex-col justify-center">
		{@render children()}
	</div>
</div>
