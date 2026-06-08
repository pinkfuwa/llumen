<script lang="ts">
	import { SearchCode, Atom, ImagePlus } from '@lucide/svelte';
	import { DropdownMenu } from 'bits-ui';
	import { t } from 'svelte-intl-precompile';
	import { effective, overridingMode } from './state.svelte';
	import { ChatMode as Mode } from '$lib/api/types';

	const itemStyle =
		'flex items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden select-none cursor-pointer duration-150 hover:bg-interactive-hover';

	function setMode(nextMode: Mode) {
		overridingMode.val = nextMode;
	}

	const searchDisabled = $derived(!effective.allowMode.search_enabled);
	const deepResearchDisabled = $derived(!effective.allowMode.deep_research);
	const mediaDisabled = $derived(!effective.allowMode.media_gen);
</script>

<div class="pt-1">
	<DropdownMenu.Separator class="mx-1 mb-1 h-px bg-border/80" />
	<div class="space-y-1">
		<DropdownMenu.Item
			class="{itemStyle} aria-disabled:opacity-50 data-disabled:opacity-50 data-[active=true]:bg-interactive-selection data-[active=true]:text-primary"
			onSelect={() => setMode(Mode.Search)}
			disabled={searchDisabled}
			data-active={effective.mode === Mode.Search ? 'true' : 'false'}
		>
			<SearchCode class="size-4" />
			<span>{$t('chat.model_mode.search')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="{itemStyle} aria-disabled:opacity-50 data-disabled:opacity-50 data-[active=true]:bg-interactive-selection data-[active=true]:text-primary"
			onSelect={() => setMode(Mode.Research)}
			disabled={deepResearchDisabled}
			data-active={effective.mode === Mode.Research ? 'true' : 'false'}
		>
			<Atom class="size-4" />
			<span>{$t('chat.model_mode.deep_research')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="{itemStyle} aria-disabled:opacity-50 data-disabled:opacity-50 data-[active=true]:bg-interactive-selection data-[active=true]:text-primary"
			onSelect={() => setMode(Mode.Media)}
			disabled={mediaDisabled}
			data-active={effective.mode === Mode.Media ? 'true' : 'false'}
		>
			<ImagePlus class="size-4" />
			<span>{$t('chat.model_mode.media')}</span>
		</DropdownMenu.Item>
	</div>
</div>
