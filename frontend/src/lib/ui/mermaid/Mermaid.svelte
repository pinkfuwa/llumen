<script lang="ts">
	import { render } from './mermaid';
	import { t } from 'svelte-intl-precompile';
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
		error = null;
		if (incremental) {
			svg = null;
			return;
		}

		let stopped = false;

		render(text)
			.then((result) => {
				if (stopped) return;
				error = null;
				svg = result;
			})
			.catch((cause: unknown) => {
				if (stopped) return;
				svg = null;
				error = cause instanceof Error ? cause.message : String(cause);
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
</script>

<div
	bind:this={containerEl}
	class="relative overflow-hidden rounded-md border border-border bg-card p-2 data-focus:ring-4 data-focus:ring-ring"
	style="height: {cssContainerHeight}"
	style:--mermaid-border="var(--border)"
	role="group"
	data-focus={zoomableFocused ? '' : undefined}
>
	{#if !incremental && error != null}
		<div class="flex h-full w-full flex-col items-center justify-center p-6">
			<div class="p-2 text-xl font-semibold text-destructive">{$t('mermaid.error')}</div>

			<div class="text-ellipsis whitespace-pre-wrap">
				{error}
			</div>
		</div>
	{:else if !incremental && svg}
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
	{/if}
</div>
