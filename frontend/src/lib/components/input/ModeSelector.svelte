<script lang="ts">
	import { ChatMode as Mode } from '$lib/api/types';
	import { SearchCode, Atom } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { DropdownMenu } from 'bits-ui';

	let {
		value = $bindable(Mode.Normal) as Mode,
		disabled = false
	}: { value: Mode; disabled?: boolean } = $props();

	function setMode(nextMode: Mode) {
		value = value === nextMode ? Mode.Normal : nextMode;
	}

	function isActive(mode: Mode) {
		return value === mode;
	}
</script>

<div class="pt-1">
	<DropdownMenu.Separator class="mx-1 mb-1 h-px bg-outline/80" />
	<div class="space-y-1">
		<DropdownMenu.Item
			class={`flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover ${isActive(Mode.Search) ? 'bg-primary text-text-hover' : ''}`}
			onSelect={() => setMode(Mode.Search)}
			disabled={disabled}
			data-active={isActive(Mode.Search) ? 'true' : 'false'}
		>
			<SearchCode class="size-4" />
			<span>{$_('chat.model_mode.search')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class={`flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover ${isActive(Mode.Research) ? 'bg-primary text-text-hover' : ''}`}
			onSelect={() => setMode(Mode.Research)}
			disabled={disabled}
			data-active={isActive(Mode.Research) ? 'true' : 'false'}
		>
			<Atom class="size-4" />
			<span>{$_('chat.model_mode.deep_research')}</span>
		</DropdownMenu.Item>
	</div>
</div>
