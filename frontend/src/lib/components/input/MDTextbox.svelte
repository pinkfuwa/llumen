<script lang="ts">
	let {
		editable = $bindable(false),
		value = $bindable(''),
		placeholder = '',
		disabled = false,
		onsubmit = undefined as undefined | (() => void)
	} = $props();

	import { default as Markdown } from '$lib/components/markdown/Root.svelte';
	import { submitOnEnter } from '$lib/preference';
	import { onStartTyping } from '@sv-use/core';
	import { get } from 'svelte/store';

	let input = $state<null | HTMLElement>(null);

	onStartTyping(() => {
		if (input !== document.activeElement) {
			editable = true;
			input?.focus();
		}
	});

	let rows = () => Math.max(2, value.split('\n').length);

	let virtualKeyboard = $state(false);
	if ('virtualKeyboard' in navigator) {
		navigator.virtualKeyboard.overlaysContent = false;

		navigator.virtualKeyboard.addEventListener('geometrychange', (event) => {
			const { width, height } = event.target.boundingRect;
			if (width > 0 || height > 0) virtualKeyboard = true;
		});
	}

	let renderValue = $state(value);
	$effect(() => {
		if (!editable) renderValue = value;
	});
</script>

<textarea
	class="editor field-sizing-content max-h-[60vh] max-w-[65vw] flex-grow resize-none overflow-auto rounded-md bg-input p-4 data-[state=hide]:hidden"
	bind:value
	{placeholder}
	rows={rows()}
	bind:this={input}
	{disabled}
	aria-label="type message"
	data-state={editable ? 'show' : 'hide'}
	onkeypress={(event) => {
		if (
			!virtualKeyboard &&
			event.key == 'Enter' &&
			!event.shiftKey &&
			get(submitOnEnter) == 'true' &&
			onsubmit
		)
			onsubmit();
	}}
></textarea>
<div
	class="new-message markdown max-h-[60vh] min-h-12 max-w-[65vw] flex-grow space-y-2 overflow-y-auto p-4 pr-2 wrap-break-word data-[state=hide]:hidden"
	data-state={editable ? 'hide' : 'show'}
>
	<Markdown source={renderValue} />
</div>
