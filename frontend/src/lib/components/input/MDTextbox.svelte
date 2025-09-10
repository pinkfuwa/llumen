<script lang="ts">
	let {
		editable = $bindable(false),
		value = $bindable(''),
		placeholder = '',
		disabled = false,
		onsubmit = undefined as undefined | (() => void)
	} = $props();

	import { default as Markdown } from '$lib/markdown/Root.svelte';
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
</script>

<textarea
	class="editor field-sizing-content max-h-[60vh] max-w-[65vw] flex-grow resize-none rounded-md p-4 focus:bg-background overflow-scroll{editable
		? ''
		: ' hidden'}"
	bind:value
	{placeholder}
	rows={rows()}
	bind:this={input}
	{disabled}
	aria-label="type message"
	onkeypress={(event) => {
		if (event.key == 'Enter' && !event.shiftKey && get(submitOnEnter) == 'true' && onsubmit)
			onsubmit();
	}}
></textarea>
{#if !editable}
	<div
		class="new-message markdown max-h-[60vh] min-h-12 max-w-[65vw] flex-grow space-y-2 overflow-y-auto pr-2 wrap-break-word"
	>
		<Markdown source={value} />
	</div>
{/if}
