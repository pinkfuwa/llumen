<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		class: className,
		children,
		selected = false,
		...rest
	}: {
		class?: string;
		children: Snippet;
		selected?: boolean;
		[key: string]: unknown;
	} = $props();
</script>

<div
	class="cursor-pointer duration-150 hover:bg-interactive-hover {className}"
	class:bg-interactive-selection={selected}
	class:text-primary={selected}
	{...rest}
>
	{@render children()}
</div>
