<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import DangerButton from '$lib/ui/DangerButton.svelte';
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
		<Button class="px-4 py-2" onclick={handleCancel}>
			{$_('chat.confirm.cancel')}
		</Button>
		<DangerButton onclick={handleConfirm}>
			{$_('chat.confirm.confirm')}
		</DangerButton>
	{/snippet}
</Modal>
