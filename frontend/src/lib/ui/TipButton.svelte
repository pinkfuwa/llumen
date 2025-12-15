<script lang="ts">
	import { Tooltip } from 'bits-ui';

	let {
		class: className,
		children,
		text,
		...restProps
	}: Tooltip.TriggerProps & { text: string } = $props();
</script>

<Tooltip.Provider>
	<Tooltip.Root delayDuration={200}>
		<Tooltip.Trigger
			class={`cursor-pointer rounded-lg border border-outline text-center text-text duration-150 not-disabled:hover:bg-primary not-disabled:hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 ${className}`}
			{...restProps}
		>
			{@render children?.()}
		</Tooltip.Trigger>
		<Tooltip.Content sideOffset={6}>
			<div class="rounded-md border border-outline bg-popup-bg p-2 text-text">
				{text}
			</div>
		</Tooltip.Content>
	</Tooltip.Root>
</Tooltip.Provider>
