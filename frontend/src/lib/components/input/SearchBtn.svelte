<script lang="ts">
	import { MessageCreateReqMode as Mode } from '$lib/api/types';
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Atom, SearchCode, ZapOff, CalendarSync } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	let { value = $bindable(Mode.Normal) as Mode, disabled = false } = $props();

	const modes = [Mode.Normal, Mode.Search, Mode.Agent, Mode.Research];
	function nextStage() {
		const nextIndex = modes.indexOf(value) + 1;
		value = modes[nextIndex % modes.length];
	}
</script>

<button
	onclick={nextStage}
	class="rounded-md bg-primary p-1{disabled ? '' : ' hover:bg-hover'}"
	aria-label="change mode"
>
	{#if value == Mode.Research}
		<Tooltip content={$_('chat.model_mode.deep')}>
			<Atom class="inline-block" />
		</Tooltip>
	{:else if value == Mode.Search}
		<Tooltip content={$_('chat.model_mode.search')}>
			<SearchCode class="inline-block" />
		</Tooltip>
	{:else if value == Mode.Agent}
		<CalendarSync class="inline-block" />
	{:else}
		<Tooltip content={$_('chat.model_mode.normal')}>
			<ZapOff class="inline-block" />
		</Tooltip>
	{/if}
</button>
