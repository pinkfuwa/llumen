<script lang="ts">
	import { render } from './mermaid';
	import { preference } from '$lib/preference/index.svelte';
	import { t } from 'svelte-intl-precompile';
	import Monochrome from '../shiki/Monochrome.svelte';
	import { getThemeStyle } from '../shiki/shiki';
	import Zoomable from '$lib/ui/Zoomable.svelte';

	let { text = '', incremental = false } = $props<{ text?: string; incremental?: boolean }>();

	let svg = $state<string | null>(null);
	let error = $state<string | null>(null);

	let containerEl = $state<HTMLDivElement>();
	let innerW = $state(0);
	let innerH = $state(0);
	let zoomableFocused = $state(false);

	const cssContainerHeight = $derived('clamp(300px, 65dvh, 600px)');

	$effect(() => {
		if (incremental) {
			svg = null;
			return;
		}

		void preference.value.theme.name;
		void preference.value.theme.dark;

		let stopped = false;

		render(text)
			.then((result) => {
				if (stopped) return;
				error = null;
				svg = result;
			})
			.catch((e) => {
				console.log('err', e);
				error = e;
			});

		return () => {
			stopped = true;
		};
	});

	$effect(() => {
		if (!svg || !containerEl) return;
		const el = containerEl;
		const id = requestAnimationFrame(() => {
			const svgElem = el.querySelector('svg');
			if (!svgElem) return;
			innerW = svgElem.clientWidth;
			innerH = svgElem.clientHeight;
		});
		return () => cancelAnimationFrame(id);
	});

	const displayText = $derived(incremental || (error == null && !svg));
	const themeStyle = $derived(displayText ? getThemeStyle(preference.value.theme) : '');
</script>

<div
	bind:this={containerEl}
	class="relative overflow-hidden rounded-md border border-border bg-card p-2 data-focus:ring-4 data-focus:ring-ring"
	style="height: {cssContainerHeight}; {themeStyle}"
	role="group"
	data-focus={zoomableFocused ? '' : undefined}
>
	{#if displayText}
		<div class="h-full overflow-y-auto">
			<Monochrome {text} />
		</div>
	{:else if error != null}
		<div class="flex h-full w-full flex-col items-center justify-center p-6">
			<div class="p-2 text-xl font-semibold text-destructive">{$t('mermaid.error')}</div>

			<div class="text-ellipsis whitespace-pre-wrap">
				{error}
			</div>
		</div>
	{:else if svg}
		<Zoomable
			contentWidth={innerW}
			contentHeight={innerH}
			bind:focused={zoomableFocused}
			class="absolute inset-0 h-full w-full select-none"
		>
			{#snippet children({ zoom, panX, panY })}
				<div
					class="pointer-events-none absolute top-0 left-0 h-full! origin-top-left"
					style="transform: translate({panX}px, {panY}px) scale({zoom})"
				>
					{@html svg}
				</div>
			{/snippet}
		</Zoomable>
	{:else}
		<span>This is a bug</span>
	{/if}
</div>
