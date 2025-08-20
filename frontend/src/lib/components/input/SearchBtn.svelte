<script lang="ts">
	type Stage = 0 | 1 | 2;

	let { value = $bindable(0) as Stage, disabled = false } = $props();
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Atom, SearchCode, ZapOff } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	function nextStage() {
		value = (value + 1) % 3;
	}
</script>

<button onclick={nextStage} class="rounded-md bg-primary p-1{disabled ? '' : ' hover:bg-hover'}">
	{#if value == 2}
		<Tooltip content={$_('chat.model_mode.deep')}>
			<Atom class="inline-block" />
		</Tooltip>
	{:else if value == 1}
		<Tooltip content={$_('chat.model_mode.search')}>
			<SearchCode class="inline-block" />
		</Tooltip>
	{:else}
		<Tooltip content={$_('chat.model_mode.normal')}>
			<ZapOff class="inline-block" />
		</Tooltip>
	{/if}
</button>
