<script lang="ts">
	import { Tooltip } from 'bits-ui';

	let {
		class: className,
		children,
		text,
		...restProps
	}: Tooltip.TriggerProps & { text: string } = $props();

	const triggerStyle =
		'cursor-pointer rounded-lg border border-border text-center text-foreground duration-150 not-disabled:hover:bg-interactive-hover focus:ring-4 focus:ring-ring focus:outline-none disabled:cursor-not-allowed disabled:opacity-60';
	const panelStyle = 'rounded-md border border-border bg-popover p-2 text-popover-foreground';
</script>

<Tooltip.Provider>
	<Tooltip.Root delayDuration={200}>
		<Tooltip.Trigger class={`${triggerStyle} ${className}`} {...restProps}>
			{@render children?.()}
		</Tooltip.Trigger>
		<Tooltip.Content sideOffset={6}>
			<div class={panelStyle}>
				{text}
			</div>
		</Tooltip.Content>
	</Tooltip.Root>
</Tooltip.Provider>
