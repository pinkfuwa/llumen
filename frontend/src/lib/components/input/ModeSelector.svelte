<script lang="ts">
	import { ChatMode as Mode } from '$lib/api/types';
	import type { ModelList } from '$lib/api/types';
	import { SearchCode, Atom, ImagePlus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { DropdownMenu } from 'bits-ui';

	let {
		value = Mode.Normal as Mode,
		modelCap = undefined as ModelList | undefined,
		onchange = undefined as ((mode: Mode) => void) | undefined
	}: { value: Mode; modelCap?: ModelList; onchange?: (mode: Mode) => void } = $props();

	const itemStyle =
		'flex items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden select-none cursor-pointer duration-150 hover:bg-interactive-hover';

	function setMode(nextMode: Mode) {
		if (onchange) {
			onchange(nextMode);
		}
	}

	function isActive(mode: Mode) {
		return value === mode;
	}

	const deepResearchDisabled = $derived(!modelCap?.deep_research);
	const mediaDisabled = $derived(!modelCap?.media_gen);
	const searchDisabled = $derived(!modelCap?.search_enabled);
</script>

<div class="pt-1">
	<DropdownMenu.Separator class="mx-1 mb-1 h-px bg-border/80" />
	<div class="space-y-1">
		<DropdownMenu.Item
			class="{itemStyle} data-disabled:opacity-50 data-[active=true]:bg-interactive-selection data-[active=true]:text-primary"
			onSelect={() => setMode(Mode.Search)}
			disabled={searchDisabled}
			aria-disabled={searchDisabled}
			data-active={isActive(Mode.Search) ? 'true' : 'false'}
		>
			<SearchCode class="size-4" />
			<span>{m['chat.model_mode.search']()}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="{itemStyle} data-disabled:opacity-50 data-[active=true]:bg-interactive-selection data-[active=true]:text-primary"
			onSelect={() => setMode(Mode.Research)}
			disabled={deepResearchDisabled}
			aria-disabled={deepResearchDisabled}
			data-active={isActive(Mode.Research) ? 'true' : 'false'}
		>
			<Atom class="size-4" />
			<span>{m['chat.model_mode.deep_research']()}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="{itemStyle} data-disabled:opacity-50 data-[active=true]:bg-interactive-selection data-[active=true]:text-primary"
			onSelect={() => setMode(Mode.Media)}
			disabled={mediaDisabled}
			aria-disabled={mediaDisabled}
			data-active={isActive(Mode.Media) ? 'true' : 'false'}
		>
			<ImagePlus class="size-4" />
			<span>{m['chat.model_mode.media']()}</span>
		</DropdownMenu.Item>
	</div>
</div>
