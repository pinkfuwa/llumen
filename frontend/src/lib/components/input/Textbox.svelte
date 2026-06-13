<script lang="ts">
	import { default as Markdown } from '$lib/components/markdown/Root.svelte';

	const editorStyle =
		'editor field-sizing-content max-h-96 flex-grow resize-none overflow-auto rounded-md bg-input p-4 data-[state=hide]:hidden md:max-h-[60vh]';
	import { onDestroy } from 'svelte';
	import { shouldSubmitOnEnter } from './submitOnEnter.svelte';
	import { stringWidthWithWrap } from '$lib/string-width';
	import { inputContent, isEditing, submitting } from './state.svelte';
	import { submit } from './state.svelte';
	import { streaming } from '$lib/api';

	let input = $state<null | HTMLElement>(null);
	let inputWidth = $state(0);
	let inputFont = $state('');

	let {
		minRow = 1,
		placeholder = ''
	}: {
		minRow?: number;
		placeholder?: string;
	} = $props();

	let disabled = $derived(submitting.val || streaming.val);

	$effect(() => {
		if (!input) return;
		const style = getComputedStyle(input);
		inputFont = `${style.fontSize} ${style.fontFamily}`;
		const ro = new ResizeObserver((entries) => {
			for (const entry of entries) {
				inputWidth = entry.contentBoxSize?.[0]?.inlineSize ?? entry.contentRect.width;
			}
		});
		ro.observe(input);
		return () => ro.disconnect();
	});

	function onKeyDown(event: KeyboardEvent) {
		const activeElement = document.activeElement;
		const { code, metaKey, ctrlKey, altKey } = event;
		if (metaKey || ctrlKey || altKey) return;

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
			isEditing.val = true;
			event.preventDefault();
		}
	}

	window.addEventListener('keydown', onKeyDown);
	onDestroy(() => window.removeEventListener('keydown', onKeyDown));

	let rows = $state(1);

	$effect(() => {
		if (!inputWidth || !inputFont || !input) return;

		const style = getComputedStyle(input);
		const horizontalPadding = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight);
		const contentWidth = inputWidth - horizontalPadding;

		rows = Math.max(
			minRow,
			Math.min(stringWidthWithWrap(inputContent.val, inputFont, contentWidth), 20)
		);
	});

	let virtualKeyboard = $state(false);
	if ('virtualKeyboard' in navigator) {
		navigator.virtualKeyboard.overlaysContent = true;

		navigator.virtualKeyboard.addEventListener('geometrychange', () => {
			virtualKeyboard = true;
			navigator.virtualKeyboard.overlaysContent = false;
		});
	}

	let renderValue = $state(inputContent.val);
	$effect(() => {
		if (!isEditing.val) renderValue = inputContent.val;
	});
</script>

<textarea
	class={editorStyle}
	bind:value={inputContent.val}
	{placeholder}
	{rows}
	bind:this={input}
	{disabled}
	aria-label="type message"
	data-state={isEditing.val ? 'show' : 'hide'}
	onkeypress={(event) => {
		if (!disabled && shouldSubmitOnEnter(event, { virtualKeyboard })) submit();
	}}
></textarea>
<div
	class="new-message markdown max-h-96 min-h-12 max-w-[65vw] flex-grow space-y-2 overflow-y-auto p-4 pr-2 wrap-break-word data-[state=hide]:hidden md:max-h-[60vh]"
	data-state={isEditing.val ? 'hide' : 'show'}
>
	<Markdown source={renderValue} />
</div>
