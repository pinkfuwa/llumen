<script>
	import { CircleCheck } from '@lucide/svelte';
	import { fade } from 'svelte/transition';

	let copied = $state(true);

	$effect(() => {
		const timeoutId = setTimeout(() => (copied = false), 500);
		return () => clearTimeout(timeoutId);
	});
</script>

{#if copied}
	<div
		class="bg-background fixed top-0 right-0 z-6 m-3 flex items-center rounded-md border border-outline bg-component-bg px-3 py-2"
		in:fade={{ duration: 150 }}
		out:fade={{ duration: 150 }}
	>
		<CircleCheck class="mr-2 inline-block" />
		Copied to clipboard
	</div>
{/if}
