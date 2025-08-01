<script lang="ts">
	import '../app.css';
	import { isLoading } from 'svelte-i18n';

	let { children } = $props();
</script>

{#if !$isLoading}
	<div class="h-full w-full bg-light text-dark">
		{@render children()}
	</div>
{/if}
