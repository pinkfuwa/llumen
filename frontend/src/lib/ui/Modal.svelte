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

	const overlayStyle =
		'fixed inset-0 z-100 backdrop-blur-md fade-in-100 fade-out-0 data-[state=closed]:animate-out data-[state=open]:animate-in';
	const contentStyle =
		'fixed top-1/2 left-1/2 z-100 w-[90%] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl border border-border bg-card p-6 font-mono text-foreground shadow-xl fade-in fade-out zoom-in zoom-out data-[state=closed]:animate-out data-[state=open]:animate-in';
	const closeStyle =
		'absolute top-4 right-4 rounded-md focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-hidden active:scale-[0.98]';
	const iconStyle = 'size-5 text-foreground';
	const titleStyle = 'mb-4 text-xl font-semibold';
	const descriptionStyle = 'mb-6 text-muted-foreground';
	const bodyStyle = 'mb-6';
	const footerStyle = 'flex justify-end gap-3';

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
		<Dialog.Overlay class={overlayStyle} onclick={handleOutsideClick} />
		<Dialog.Content class={contentStyle}>
			<Dialog.Close class={closeStyle} onclick={handleClose}>
				<div class="p-1">
					<X class={iconStyle} />
					<span class="sr-only">Close</span>
				</div>
			</Dialog.Close>

			{#if title}
				<Dialog.Title class={titleStyle}>
					{title}
				</Dialog.Title>
			{/if}

			{#if description}
				<Dialog.Description class={descriptionStyle}>
					{description}
				</Dialog.Description>
			{/if}

			{#if children}
				<div class={bodyStyle}>
					{@render children()}
				</div>
			{/if}

			{#if footer}
				<div class={footerStyle}>
					{@render footer()}
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
