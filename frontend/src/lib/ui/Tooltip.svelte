<script lang="ts">
	import { Tooltip } from 'bits-ui';
	import type { Snippet } from 'svelte';

	let {
		class: className,
		children,
		text,
		...restProps
	}: Tooltip.TriggerProps & { text: string; children: Snippet } = $props();
</script>

<Tooltip.Provider>
	<Tooltip.Root delayDuration={200}>
		<Tooltip.Trigger
			{...restProps}
			class={`cursor-pointer rounded-lg text-center text-primary duration-150 not-disabled:hover:bg-accent not-disabled:hover:text-inverse focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 ${className}`}
		>
			{@render children()}
		</Tooltip.Trigger>
		<Tooltip.Content sideOffset={6}>
			<div class="rounded-md border border-outline bg-surface-overlay p-2 text-primary">
				{text}
			</div>
		</Tooltip.Content>
	</Tooltip.Root>
</Tooltip.Provider>
