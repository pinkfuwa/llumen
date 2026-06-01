<script lang="ts">
	import { Button } from 'bits-ui';

	let {
		class: className,
		children,
		borderless = false,
		...restProps
	}: Button.RootProps & { borderless?: boolean } = $props();
</script>

<!-- hover:bg-accent hover:text-inverse not-disabled:hover:bg-accent not-disabled:hover:text-inverse focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 -->
<Button.Root
	class={`text-primary cursor-pointer rounded-lg p-2 text-center duration-150 not-disabled:hover:bg-accent not-disabled:hover:text-inverse focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 ${borderless ? '' : 'border border-outline'} ${className}`}
	{...restProps}
>
	{@render children?.()}
</Button.Root>
