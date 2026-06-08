<script lang="ts">
	import '../app.css';
	import Copy from '$lib/components/common/Copy.svelte';
	import Error from '$lib/components/common/Error.svelte';
	import '$lib/api/auth.svelte';
	import '$lib/preference/index.svelte';
	import '$lib/i18n/index.svelte';

	let { children } = $props();
</script>

{#await import('$lib/components/PwaPrompt.svelte') then { default: ReloadPrompt }}
	<ReloadPrompt />
{/await}
<Copy />
<Error />
<div class="bg-surface-base h-full w-full text-foreground">
	{@render children()}
</div>
