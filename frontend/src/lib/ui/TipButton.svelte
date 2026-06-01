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
			class={`text-primary cursor-pointer rounded-lg border border-outline text-center duration-150 not-disabled:hover:bg-accent not-disabled:hover:text-inverse focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 ${className}`}
			{...restProps}
		>
			{@render children?.()}
		</Tooltip.Trigger>
		<Tooltip.Content sideOffset={6}>
			<div class="text-primary rounded-md border border-outline bg-surface-overlay p-2">
				{text}
			</div>
		</Tooltip.Content>
	</Tooltip.Root>
</Tooltip.Provider>
