<script lang="ts">
	import { render } from './mermaid';
	import { preference } from '$lib/preference/index.svelte';
	import { t } from 'svelte-intl-precompile';
	import Monochrome from '../shiki/Monochrome.svelte';
	import { getThemeStyle } from '../shiki/shiki';

	let { text = '', incremental = false } = $props<{ text?: string; incremental?: boolean }>();

	let svg = $state<string | null>(null);
	let error = $state<string | null>(null);

	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let isDragging = $state(false);

	let containerEl = $state<HTMLDivElement>();
	let startX = 0;
	let startY = 0;

	const containerHeight = $derived('clamp(300px, 65dvh, 600px)');

	$effect(() => {
		if (incremental) {
			svg = null;
			zoom = 1;
			panX = 0;
			panY = 0;
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
			const svgW = Number(svgElem.getAttribute('width')) || svgElem.clientWidth || 0;
			const svgH = Number(svgElem.getAttribute('height')) || svgElem.clientHeight || 0;
			const cw = el.clientWidth;
			const ch = el.clientHeight;
			const fit = Math.min(cw / (svgW || 1), ch / (svgH || 1), 2);
			zoom = Math.max(0.2, fit);
			panX = (cw - svgW * fit) / 2;
			panY = (ch - svgH * fit) / 2;
		});
		return () => cancelAnimationFrame(id);
	});

	import type { HTMLAttributes } from 'svelte/elements';

	const events: Partial<HTMLAttributes<HTMLDivElement>> = {
		onwheel(e: WheelEvent) {
			e.preventDefault();
			const newZoom = Math.max(0.2, Math.min(5, zoom * (1 + -e.deltaY * 0.001)));
			const rect = containerEl!.getBoundingClientRect();
			const mx = e.clientX - rect.left;
			const my = e.clientY - rect.top;
			panX = mx - ((mx - panX) / zoom) * newZoom;
			panY = my - ((my - panY) / zoom) * newZoom;
			zoom = newZoom;
		},
		onpointerdown(e: PointerEvent) {
			isDragging = true;
			startX = e.clientX - panX;
			startY = e.clientY - panY;
			(e.target as HTMLElement).setPointerCapture(e.pointerId);
		},
		onpointermove(e: PointerEvent) {
			if (!isDragging) return;
			panX = e.clientX - startX;
			panY = e.clientY - startY;
		},
		onpointercancel() {
			isDragging = false;
		},
		onpointerup() {
			isDragging = false;
		},

		ondblclick() {
			zoom = 1;
			panX = 0;
			panY = 0;
		}
	};

	const displayText = $derived(incremental || (error == null && !svg));
	const themeStyle = $derived(displayText ? getThemeStyle(preference.value.theme) : '');
</script>

<div
	bind:this={containerEl}
	class="relative overflow-hidden rounded-md border border-border bg-card p-2"
	style="height: {containerHeight}; {themeStyle}"
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
		<div
			class="absolute inset-0 h-full w-full touch-none overflow-hidden select-none"
			style="cursor: {isDragging ? 'grabbing' : 'grab'}"
			role="img"
			tabindex="-1"
			{...events}
		>
			<div
				class="pointer-events-none absolute top-0 left-0 h-full! origin-center"
				style="transform: translate({panX}px, {panY}px) scale({zoom})"
			>
				{@html svg}
			</div>
		</div>
	{:else}
		<span>This is a bug</span>
	{/if}
</div>
