<script lang="ts">
	import Modal from '$lib/ui/Modal.svelte';
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
</script>

<Modal
	bind:open
	title={$_('chat.confirm.title')}
	description={$_('chat.confirm.message')}
	closeOnOutsideClick={true}
	onClose={handleConfirm}
>
	{#snippet footer()}
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
	{/snippet}
</Modal>
