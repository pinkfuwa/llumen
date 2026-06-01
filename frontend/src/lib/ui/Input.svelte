<script lang="ts">
	import { Label } from 'bits-ui';
	import type { ClassValue, HTMLInputAttributes } from 'svelte/elements';

	let {
		class: className,
		labelClass = '',
		id,
		children,
		value = $bindable(),
		...attr
	}: HTMLInputAttributes & { labelClass?: ClassValue } = $props();

	const labelStyle = 'mb-3 block min-w-8 text-sm font-medium text-foreground';
	const inputStyle =
		'block w-full rounded-lg border border-border bg-input p-2.5 text-sm text-foreground focus:ring-2 focus:ring-ring focus:outline-none';
</script>

<Label.Root for={id} class={`${labelStyle} ${labelClass}`}>
	{@render children?.()}
</Label.Root>
<input {id} bind:value class={`${inputStyle} ${className}`} {...attr} />
