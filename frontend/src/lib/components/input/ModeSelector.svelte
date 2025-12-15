<script lang="ts">
	import { ChatMode as Mode } from '$lib/api/types';
	import { Atom, SearchCode, ZapOff } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import TipButton from '$lib/ui/TipButton.svelte';

	let {
		value = $bindable(Mode.Normal) as Mode,
		disabled = false,
		limited = false
	}: { value: Mode; disabled?: boolean; limited?: boolean } = $props();

	// TODO: enable Mode.Research when ready
	const modes = [Mode.Normal, Mode.Search, Mode.Research];
	function nextStage() {
		const nextIndex = modes.indexOf(value) + 1;
		value = modes[nextIndex % modes.length];
	}

	const tipText: Record<Mode, string> = $derived({
		[Mode.Normal]: $_('chat.model_mode.normal'),
		[Mode.Search]: $_('chat.model_mode.search'),
		[Mode.Research]: $_('chat.model_mode.deep')
	});

	$effect(() => {
		if (limited) value = Mode.Normal;
	});
</script>

<TipButton
	onclick={nextStage}
	class="aspect-square h-full shrink-0"
	disabled={disabled || limited}
	aria-label="change mode"
	text={tipText[value]}
>
	{#if value == Mode.Research}
		<Atom class="inline-block" />
	{:else if value == Mode.Search}
		<SearchCode class="inline-block" />
	{:else}
		<ZapOff class="inline-block" />
	{/if}
</TipButton>
