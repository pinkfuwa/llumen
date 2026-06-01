<script lang="ts">
	import { Button } from 'bits-ui';

	let {
		class: className,
		children,
		borderless = false,
		...restProps
	}: Button.RootProps & { borderless?: boolean } = $props();

	const base =
		'cursor-pointer rounded-lg p-2 text-center text-foreground duration-150 not-disabled:hover:bg-interactive-hover focus:ring-4 focus:ring-ring focus:outline-none disabled:cursor-not-allowed disabled:opacity-60';
	const bordered = 'border border-border';
</script>

<Button.Root class={`${base} ${borderless ? '' : bordered} ${className}`} {...restProps}>
	{@render children?.()}
</Button.Root>
