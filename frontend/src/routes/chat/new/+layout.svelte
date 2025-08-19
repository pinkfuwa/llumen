<script>
	let { children } = $props();
	import { _ } from 'svelte-i18n';
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>
{@render children()}
