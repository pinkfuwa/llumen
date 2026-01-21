<script lang="ts">
	import { DropdownMenu } from 'bits-ui';
	import type { Snippet } from 'svelte';

	interface Props {
		open?: boolean;
		children: Snippet;
		trigger: Snippet;
		align?: 'start' | 'center' | 'end';
		side?: 'top' | 'bottom' | 'left' | 'right';
	}

	let {
		open = $bindable(false),
		children,
		trigger,
		align = 'start',
		side = 'bottom'
	}: Props = $props();
</script>

<DropdownMenu.Root bind:open>
	<DropdownMenu.Trigger
		class="cursor-pointer rounded-lg border border-outline p-2 text-center text-text duration-150 not-disabled:hover:bg-primary not-disabled:hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
	>
		{@render trigger()}
	</DropdownMenu.Trigger>
	<DropdownMenu.Portal>
		<DropdownMenu.Content
			class="z-20 min-w-48 rounded-xl border border-outline bg-popup-bg p-1 text-text shadow-lg outline-hidden data-[side=bottom]:translate-y-1 data-[side=bottom]:slide-in-from-top-2 data-[side=top]:-translate-y-1 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95"
			sideOffset={6}
			{align}
			{side}
		>
			{@render children()}
		</DropdownMenu.Content>
	</DropdownMenu.Portal>
</DropdownMenu.Root>
