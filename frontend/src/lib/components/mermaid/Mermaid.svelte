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

	let dragging = $state(false);
	let containerEl = $state<HTMLDivElement>();
	let touchCount = 0;

	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);

	let startX = 0;
	let startY = 0;

	let deltaX = 0;
	let deltaY = 0;

	let originalZoom = 0;
	let touchGap = 0;

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

	function getStablizedTouches(touches: TouchList) {
		const result = [];
		for (let i = 0; i < touches.length; i++) {
			result.push(touches[i]);
		}
		return result.sort((a, b) => a.clientX - b.clientX);
	}

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
		ontouchstart(e) {
			touchCount = e.touches.length;
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

			const touches = getStablizedTouches(e.touches);

			if (touches.length == 2) {
				touchGap =
					(touches[0].clientX - touches[1].clientX) ** 2 +
					(touches[0].clientY - touches[1].clientY) ** 2;
				originalZoom = zoom;
			} else if (touches.length == 1) {
				dragging = true;
				startX = touches[0].clientX - panX;
				startY = touches[0].clientY - panY;
			}
			e.preventDefault();
		},
		ontouchmove(e) {
			const touches = getStablizedTouches(e.touches);
			deltaX += touches[0].clientX - startX;
			deltaY += touches[0].clientY - startY;
			if (!dragging) return;

			if (touches.length == 2) {
				const newGap =
					(touches[0].clientX - touches[1].clientX) ** 2 +
					(touches[0].clientY - touches[1].clientY) ** 2;
				zoom = originalZoom * Math.sqrt(newGap / touchGap);
			} else if (touches.length == 1) {
				dragging = true;
				panX = touches[0].clientX - startX;
				panY = touches[0].clientY - startY;
			}
			// from https://w3c.github.io/touch-events/#event-touchmove
			// prevenDefault does not stop mouse event.
			e.preventDefault();
		},
		ontouchend(e) {
			touchCount -= e.touches.length;
			dragging = false;
			// preventDefault cancel the onclick event, which is unwanted.
		},
		onpointerdown(e) {
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

			dragging = true;
			startX = e.clientX - panX;
			startY = e.clientY - panY;
			(e.target as HTMLElement).setPointerCapture(e.pointerId);
		},
		onpointermove(e) {
			if (touchCount > 0) return;
			deltaX += e.movementX;
			deltaY += e.movementY;

			if (!dragging) return;
			panX = e.clientX - startX;
			panY = e.clientY - startY;
		},
		onpointerleave() {
			focus = false;
		},
		onpointercancel() {
			dragging = false;
		},
		onpointerup() {
			dragging = false;
		},
		onclick() {
			if (Math.max(Math.abs(deltaX), Math.abs(deltaY)) < deadZone) {
				focus = !focus;
			}
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
			class="absolute inset-0 h-full w-full cursor-grab overflow-hidden select-none data-dragging:cursor-grabbing data-focus:touch-none"
			style="cursor: {dragging ? 'grabbing' : 'grab'}"
			role="img"
			tabindex="-1"
			data-focus={focus ? '' : undefined}
			data-dragging={dragging ? '' : undefined}
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
