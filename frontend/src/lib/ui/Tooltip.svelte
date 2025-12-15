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
			class={`cursor-pointer rounded-lg text-center text-text duration-150 not-disabled:hover:bg-primary not-disabled:hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 ${className}`}
		>
			{@render children()}
		</Tooltip.Trigger>
		<Tooltip.Content sideOffset={6}>
			<div class="rounded-md border border-outline bg-popup-bg p-2 text-text">
				{text}
			</div>
		</Tooltip.Content>
	</Tooltip.Root>
</Tooltip.Provider>
