<script lang="ts">
	import { render } from './mermaid';
	import { preference } from '$lib/preference/index.svelte';
	import Code from '../shiki/Code.svelte';

	let { text = '', closed = false } = $props<{ text?: string; closed?: boolean }>();

	let svg = $state<string | null>(null);
	let error = $state<string | null>(null);
	let rendering = $state(false);

	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let isDragging = $state(false);

	let containerEl = $state<HTMLDivElement>();
	let startX = 0;
	let startY = 0;

	const containerHeight = $derived('clamp(300px, 65dvh, 600px)');

	$effect(() => {
		if (!closed || !text) {
			svg = null;
			error = null;
			rendering = false;
			zoom = 1;
			panX = 0;
			panY = 0;
			return;
		}

		const timer = setTimeout(() => {
			rendering = true;
			error = null;
			render(text)
				.then((result) => {
					svg = result;
				})
				.catch((e) => {
					error = e.message;
				})
				.finally(() => {
					rendering = false;
				});
		}, 300);

		return () => clearTimeout(timer);
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

	function handleWheel(e: WheelEvent) {
		e.preventDefault();
		const newZoom = Math.max(0.2, Math.min(5, zoom * (1 + -e.deltaY * 0.001)));
		const rect = containerEl!.getBoundingClientRect();
		const mx = e.clientX - rect.left;
		const my = e.clientY - rect.top;
		panX = mx - ((mx - panX) / zoom) * newZoom;
		panY = my - ((my - panY) / zoom) * newZoom;
		zoom = newZoom;
	}

	function handlePointerDown(e: PointerEvent) {
		isDragging = true;
		startX = e.clientX - panX;
		startY = e.clientY - panY;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function handlePointerMove(e: PointerEvent) {
		if (!isDragging) return;
		panX = e.clientX - startX;
		panY = e.clientY - startY;
	}

	function handlePointerUp() {
		isDragging = false;
	}

	function handleDblClick() {
		zoom = 1;
		panX = 0;
		panY = 0;
	}
</script>

<div
	bind:this={containerEl}
	class="mermaid-container rounded-md border border-border"
	style="height: {containerHeight}"
>
	{#if !closed || rendering || (!svg && !error)}
		<div class="mermaid-scroll">
			<Code {text} />
		</div>
	{:else if error}
		<div class="mermaid-error">
			<span class="mermaid-error-icon">△</span>
			<span>{error}</span>
		</div>
	{:else}
		<!-- svelte-ignore a11y_no_static_element_interactions a11y_no_noninteractive_tabindex -->
		<div
			class="mermaid-zoom-layer"
			style="cursor: {isDragging ? 'grabbing' : 'grab'}"
			role="img"
			tabindex="-1"
			onwheel={handleWheel}
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={handlePointerUp}
			onpointercancel={handlePointerUp}
			ondblclick={handleDblClick}
		>
			<div class="mermaid-svg" style="transform: translate({panX}px, {panY}px) scale({zoom})">
				{@html svg}
			</div>
		</div>
	{/if}
</div>
