<script lang="ts">
	import { Button } from 'bits-ui';

	let { class: className, children, ...restProps }: Button.RootProps = $props();
</script>

<Button.Root
	class={`
		rounded-lg
		border
		border-outline
		text-center
		text-text
		duration-150
		not-disabled:hover:bg-primary
		not-disabled:hover:text-text-hover
		focus:ring-4
		focus:ring-outline
		focus:outline-none
		disabled:cursor-not-allowed
		disabled:opacity-60
		${className}
	`}
	{...restProps}
>
	{@render children?.()}
</Button.Root>
