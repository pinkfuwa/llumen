<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ClassValue } from 'svelte/elements';

	let {
		class: className,
		children,
		onclick,
		type = 'button',
		...rest
	}: {
		class?: ClassValue;
		children: Snippet;
		onclick?: (event: MouseEvent) => void;
		type?: 'button' | 'submit' | 'reset';
		[key: string]: unknown;
	} = $props();
</script>

<button
	{type}
	{onclick}
	class="cursor-pointer rounded-md bg-destructive px-4 py-2 text-destructive-foreground transition-colors hover:bg-destructive/80 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-60 {className}"
	{...rest}
>
	{@render children()}
</button>
