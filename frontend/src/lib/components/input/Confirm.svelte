<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { X } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	interface Props {
		open: boolean;
		onconfirm: () => void;
		oncancel?: () => void;
	}

	let { open = $bindable(), onconfirm, oncancel }: Props = $props();

	function handleConfirm() {
		open = false;
		onconfirm();
	}

	function handleCancel() {
		open = false;
		if (oncancel) oncancel();
	}

	function handleOutsideClick() {
		handleConfirm();
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
				onclick={handleCancel}
			>
				<div class="p-1">
					<X class="text-foreground size-5" />
					<span class="sr-only">Close</span>
				</div>
			</Dialog.Close>

			<Dialog.Title class="mb-4 text-xl font-semibold">
				{$_('chat.confirm.title')}
			</Dialog.Title>

			<Dialog.Description class="mb-6 text-text/80">
				{$_('chat.confirm.message')}
			</Dialog.Description>

			<div class="flex justify-end gap-3">
				<button
					onclick={handleCancel}
					class="rounded-md border border-outline bg-transparent px-4 py-2 transition-colors hover:bg-primary focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
				>
					{$_('chat.confirm.cancel')}
				</button>
				<button
					onclick={handleConfirm}
					class="rounded-md border border-outline bg-primary px-4 py-2 transition-colors hover:bg-primary/80 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
				>
					{$_('chat.confirm.confirm')}
				</button>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
