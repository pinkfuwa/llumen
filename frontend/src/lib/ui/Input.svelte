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
</script>

<Label.Root for={id} class={`mb-3 block min-w-8 text-sm font-medium text-text ${labelClass}`}>
	{@render children?.()}
</Label.Root>
<input
	{id}
	bind:value
	class={`block w-full rounded-lg border border-outline bg-input p-2.5 text-sm text-gray-900 focus:ring-2 focus:ring-outline focus:outline-none ${className}`}
	{...attr}
/>
