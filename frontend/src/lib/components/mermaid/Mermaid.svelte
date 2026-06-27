<script lang="ts">
	import { render } from './mermaid';
	import { preference } from '$lib/preference/index.svelte';
	import { t } from 'svelte-intl-precompile';
	import Monochrome from '../shiki/Monochrome.svelte';
	import { getThemeStyle } from '../shiki/shiki';
	import type { HTMLAttributes } from 'svelte/elements';

	let { text = '', incremental = false } = $props<{ text?: string; incremental?: boolean }>();

	const deadZone = 35;

	let svg = $state<string | null>(null);
	let error = $state<string | null>(null);

	let isDragging = $state(false);
	let containerEl = $state<HTMLDivElement>();

	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);

	let startX = 0;
	let startY = 0;

	let deltaX = 0;
	let deltaY = 0;

	let innerW = 0;
	let innerH = 0;

	let containerW = 0;
	let containerH = 0;

	const cssContainerHeight = $derived('clamp(300px, 65dvh, 600px)');

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
			innerW = svgElem.clientWidth;
			innerH = svgElem.clientHeight;
			containerW = el.clientWidth;
			containerH = el.clientHeight;
			const fit = Math.min(containerW / (innerW || 1), containerH / (innerH || 1), 2);
			zoom = Math.max(0.2, fit);
			panX = (containerW - innerW * fit) / 2;
			panY = (containerW - innerH * fit) / 2;
		});
		return () => cancelAnimationFrame(id);
	});

	$effect(() => {
		function limitRange(value: number, min: number, max: number, gap: number): number {
			return Math.max(min - gap, Math.min(max + gap, value));
		}

		panX = limitRange(panX, -innerW * zoom, containerW, -20);
		panY = limitRange(panY, -innerH * zoom, containerH, -20);
	});

	let focus = $state(false);

	const events: Partial<HTMLAttributes<HTMLDivElement>> = {
		onwheel(e: WheelEvent) {
			if (!focus) return;
			e.preventDefault();
			const newZoom = Math.max(0.32, Math.min(5, zoom * (1 + -e.deltaY * 0.001)));
			const rect = containerEl!.getBoundingClientRect();
			const mx = e.clientX - rect.left;
			const my = e.clientY - rect.top;
			panX = mx - ((mx - panX) / zoom) * newZoom;
			panY = my - ((my - panY) / zoom) * newZoom;
			zoom = newZoom;
		},
		onpointerdown(e: PointerEvent) {
			deltaX = 0;
			deltaY = 0;

			if (!focus) return;
			const svgElem = containerEl!.querySelector('svg');
			if (svgElem) {
				innerW = svgElem.clientWidth;
				innerH = svgElem.clientHeight;
			}

			containerW = containerEl!.clientWidth;
			containerH = containerEl!.clientHeight;

			isDragging = true;
			startX = e.clientX - panX;
			startY = e.clientY - panY;
			(e.target as HTMLElement).setPointerCapture(e.pointerId);
		},
		onpointermove(e: PointerEvent) {
			deltaX += e.movementX;
			deltaY += e.movementY;

			if (!isDragging) return;
			panX = e.clientX - startX;
			panY = e.clientY - startY;
		},
		onpointerleave() {
			focus = false;
		},
		onpointercancel() {
			isDragging = false;
		},
		onclick() {
			if (Math.max(Math.abs(deltaX), Math.abs(deltaY)) < deadZone) {
				focus = !focus;
			}
		},
		onpointerup() {
			isDragging = false;
		}
	};

	const displayText = $derived(incremental || (error == null && !svg));
	const themeStyle = $derived(displayText ? getThemeStyle(preference.value.theme) : '');
</script>

<div
	bind:this={containerEl}
	class="relative overflow-hidden rounded-md border border-border bg-card p-2 data-focus:ring-4 data-focus:ring-ring"
	style="height: {cssContainerHeight}; {themeStyle}"
	role="group"
	data-focus={focus ? '' : undefined}
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
				class="pointer-events-none absolute top-0 left-0 h-full! origin-top-left"
				style="transform: translate({panX}px, {panY}px) scale({zoom})"
			>
				{@html svg}
			</div>
		</div>
	{:else}
		<span>This is a bug</span>
	{/if}
</div>
