<script lang="ts">
	import { ChatMode as Mode } from '$lib/api/types';
	import type { ModelList } from '$lib/api/types';
	import { SearchCode, Atom, ImagePlus } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { DropdownMenu } from 'bits-ui';

	let {
		value = $bindable(Mode.Normal) as Mode,
		modelCap = undefined
	}: { value: Mode; modelCap?: ModelList } = $props();

	function setMode(nextMode: Mode) {
		value = value === nextMode ? Mode.Normal : nextMode;
	}

	function isActive(mode: Mode) {
		return value === mode;
	}

	const deepResearchDisabled = $derived(modelCap != null && !modelCap.tool);
	const mediaDisabled = $derived(modelCap != null && !modelCap.media_gen);
</script>

<div class="pt-1">
	<DropdownMenu.Separator class="mx-1 mb-1 h-px bg-outline/80" />
	<div class="space-y-1">
		<DropdownMenu.Item
			class={`flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover data-disabled:opacity-50 ${isActive(Mode.Search) ? 'bg-primary text-text-hover' : ''}`}
			onSelect={() => setMode(Mode.Search)}
			data-active={isActive(Mode.Search) ? 'true' : 'false'}
		>
			<SearchCode class="size-4" />
			<span>{$_('chat.model_mode.search')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class={`flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover data-disabled:opacity-50 ${isActive(Mode.Research) ? 'bg-primary text-text-hover' : ''}`}
			onSelect={() => setMode(Mode.Research)}
			disabled={deepResearchDisabled}
			aria-disabled={deepResearchDisabled}
			data-active={isActive(Mode.Research) ? 'true' : 'false'}
		>
			<Atom class="size-4" />
			<span>{$_('chat.model_mode.deep_research')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class={`flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover data-disabled:opacity-50 ${isActive(Mode.Media) ? 'bg-primary text-text-hover' : ''}`}
			onSelect={() => setMode(Mode.Media)}
			disabled={mediaDisabled}
			aria-disabled={mediaDisabled}
			data-active={isActive(Mode.Media) ? 'true' : 'false'}
		>
			<ImagePlus class="size-4" />
			<span>Media</span>
		</DropdownMenu.Item>
	</div>
</div>
