<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		class: className,
		children,
		...rest
	}: {
		class?: string;
		children: Snippet;
	} = $props();
</script>

<span class="rounded-md bg-interactive-selection px-2 py-0.5 {className}" {...rest}>
	{@render children()}
</span>
