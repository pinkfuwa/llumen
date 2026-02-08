<script lang="ts">
	let {
		isEditing = $bindable(false),
		value = $bindable(''),
		placeholder = '',
		disabled = false,
		onsubmit,
		minRow = 1
	}: {
		// True shows the textarea editor; false shows markdown preview.
		isEditing: boolean;
		value: string;
		placeholder: string;
		disabled: boolean;
		onsubmit?: () => void;
		minRow?: number;
	} = $props();

	import { default as Markdown } from '$lib/components/markdown/Root.svelte';
	import { onDestroy } from 'svelte';
	import { shouldSubmitOnEnter } from './submitOnEnter';

	let input = $state<null | HTMLElement>(null);

	function onKeyDown(event: KeyboardEvent) {
		const activeElement = document.activeElement;
		const { code, metaKey, ctrlKey, altKey } = event;
		if (metaKey || ctrlKey || altKey) return false;

		if (
			activeElement &&
			(activeElement.tagName == 'INPUT' ||
				activeElement.tagName == 'TEXTAREA' ||
				activeElement.hasAttribute('contenteditable'))
		)
			return;

		if (code == 'Enter') return;
		if (!code.startsWith('Key') && !code.startsWith('Digit')) return;

		if (input !== document.activeElement) {
			input?.focus();
			isEditing = true;
			event.preventDefault();
		}
	}

	window.addEventListener('keydown', onKeyDown);
	onDestroy(() => window.removeEventListener('keydown', onKeyDown));

	let rows = () => Math.max(minRow, value.split('\n').length);

	let virtualKeyboard = $state(false);
	if ('virtualKeyboard' in navigator) {
		navigator.virtualKeyboard.overlaysContent = true;

		navigator.virtualKeyboard.addEventListener('geometrychange', (event) => {
			virtualKeyboard = true;
			navigator.virtualKeyboard.overlaysContent = false;
		});
	}

	let renderValue = $state(value);
	$effect(() => {
		if (!isEditing) renderValue = value;
	});
</script>

<!-- TODO: replace max-h-96 with calc(... keyboard-inset-height) -->
<textarea
	class="editor field-sizing-content max-h-96 flex-grow resize-none overflow-auto rounded-md bg-input p-4 data-[state=hide]:hidden md:max-h-[60vh]"
	bind:value
	{placeholder}
	rows={rows()}
	bind:this={input}
	{disabled}
	aria-label="type message"
	data-state={isEditing ? 'show' : 'hide'}
	onkeypress={(event) => {
		if (shouldSubmitOnEnter(event, { virtualKeyboard }) && onsubmit) onsubmit();
	}}
></textarea>
<div
	class="new-message markdown max-h-96 min-h-12 max-w-[65vw] flex-grow space-y-2 overflow-y-auto p-4 pr-2 wrap-break-word data-[state=hide]:hidden md:max-h-[60vh]"
	data-state={isEditing ? 'hide' : 'show'}
>
	<Markdown source={renderValue} />
</div>
