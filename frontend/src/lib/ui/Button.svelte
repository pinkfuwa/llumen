<script lang="ts">
	import { Button } from 'bits-ui';

	let {
		class: className,
		children,
		borderless = false,
		...restProps
	}: Button.RootProps & { borderless?: boolean } = $props();
</script>

<!-- hover:bg-primary hover:text-text-hover not-disabled:hover:bg-primary not-disabled:hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 -->
<Button.Root
	class={`cursor-pointer rounded-lg p-2 text-center text-text duration-150 not-disabled:hover:bg-primary not-disabled:hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 ${borderless ? '' : 'border border-outline'} ${className}`}
	{...restProps}
>
	{@render children?.()}
</Button.Root>
