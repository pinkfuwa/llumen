<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { X } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		open: boolean;
		title?: string;
		description?: string;
		onClose?: () => void;
		children?: Snippet;
		footer?: Snippet;
		closeOnOutsideClick?: boolean;
	}

	let {
		open = $bindable(),
		title,
		description,
		onClose,
		children,
		footer,
		closeOnOutsideClick = false
	}: Props = $props();

	function handleClose() {
		open = false;
		if (onClose) onClose();
	}

	function handleOutsideClick() {
		if (closeOnOutsideClick) {
			handleClose();
		}
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay
			class="fixed inset-0 z-100 backdrop-blur-md fade-in-100 fade-out-0 data-[state=closed]:animate-out data-[state=open]:animate-in"
			onclick={handleOutsideClick}
		/>
		<Dialog.Content
			class="fixed top-1/2 left-1/2 z-100 w-[90%] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl border border-outline bg-popup-bg p-6 font-mono text-text shadow-xl fade-in fade-out zoom-in zoom-out data-[state=closed]:animate-out data-[state=open]:animate-in"
		>
			<Dialog.Close
				class="focus-visible:ring-foreground focus-visible:ring-offset-background absolute top-4 right-4 rounded-md focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden active:scale-[0.98]"
				onclick={handleClose}
			>
				<div class="p-1">
					<X class="text-foreground size-5" />
					<span class="sr-only">Close</span>
				</div>
			</Dialog.Close>

			{#if title}
				<Dialog.Title class="mb-4 text-xl font-semibold">
					{title}
				</Dialog.Title>
			{/if}

			{#if description}
				<Dialog.Description class="mb-6 text-text/80">
					{description}
				</Dialog.Description>
			{/if}

			{#if children}
				<div class="mb-6">
					{@render children()}
				</div>
			{/if}

			{#if footer}
				<div class="flex justify-end gap-3">
					{@render footer()}
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
