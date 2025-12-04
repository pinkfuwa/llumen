<script lang="ts">
	import { ChatMode as Mode } from '$lib/api/types';
	import { Atom, SearchCode, ZapOff } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';
	import Tooltip from '../common/Tooltip.svelte';

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

	$effect(() => {
		if (limited) value = Mode.Normal;
	});
</script>

<Button
	onclick={nextStage}
	class="aspect-square h-full shrink-0"
	disabled={disabled || limited}
	aria-label="change mode"
>
	{#if value == Mode.Research}
		<Tooltip text={$_('chat.model_mode.deep')}>
			<Atom class="inline-block" />
		</Tooltip>
	{:else if value == Mode.Search}
		<Tooltip text={$_('chat.model_mode.search')}>
			<SearchCode class="inline-block" />
		</Tooltip>
	{:else}
		<Tooltip text={$_('chat.model_mode.normal')}>
			<ZapOff class="inline-block" />
		</Tooltip>
	{/if}
</Button>
