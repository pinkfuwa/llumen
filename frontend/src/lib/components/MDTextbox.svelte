<script lang="ts">
	let {
		editable = $bindable(false),
		value = $bindable(''),
		placeholder = '',
		disabled = false
	} = $props();

	import { default as Markdown } from './markdown/Root.svelte';
	import { onStartTyping } from '@sv-use/core';

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
	class="editor field-sizing-content max-h-[60vh] max-w-[65vw] flex-grow resize-none overflow-scroll{editable
		? ''
		: ' hidden'}"
	bind:value
	{placeholder}
	rows={rows()}
	bind:this={input}
	{disabled}
></textarea>
{#if !editable}
	<div
		class="new-message markdown max-h-[60vh] min-h-12 max-w-[65vw] flex-grow space-y-2 overflow-y-auto pr-2 wrap-break-word"
	>
		<Markdown source={value} />
	</div>
{/if}
