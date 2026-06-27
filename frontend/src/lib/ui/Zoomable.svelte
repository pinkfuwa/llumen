<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		contentWidth: number;
		contentHeight: number;
		minZoom?: number;
		maxZoom?: number;
		scrollSensitivity?: number;
		visibleMargin?: number;
		autoFit?: boolean;
		focused?: boolean;
		class?: string;
		children: Snippet<
			[{ zoom: number; panX: number; panY: number; focused: boolean; dragging: boolean }]
		>;
	}

	let {
		contentWidth,
		contentHeight,
		minZoom = 0.32,
		maxZoom = 15,
		scrollSensitivity = 0.001,
		visibleMargin = 20,
		autoFit: shouldAutoFit = true,
		focused = $bindable(false),
		class: className = '',
		children
	}: Props = $props();

	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let containerEl = $state<HTMLDivElement>();
	let containerWidth = $state(0);
	let containerHeight = $state(0);

	let panning = false;
	let pinching = false;
	let panAnchorX = 0;
	let panAnchorY = 0;
	let movementSum = 0;
	let gestureStartX = 0;
	let gestureStartY = 0;
	const deadZone = 35;

	let touchActive = 0;
	let lastTouchEnd = 0;

	let pinchZoom = 0;
	let pinchPanX = 0;
	let pinchPanY = 0;
	let pinchGap = 0;
	let pinchMidX = 0;
	let pinchMidY = 0;

	let dragging = $derived(panning || pinching);

	$effect(() => {
		const el = containerEl;
		if (!el) return;
		let active = true;
		const measure = () => {
			if (!active) return;
			containerWidth = el.clientWidth;
			containerHeight = el.clientHeight;
		};
		const ro = new ResizeObserver(measure);
		ro.observe(el);
		measure();
		return () => {
			active = false;
			ro.disconnect();
		};
	});

	$effect(() => {
		if (!shouldAutoFit || !contentWidth || !contentHeight || !containerWidth || !containerHeight)
			return;
		const fit = Math.min(
			containerWidth / (contentWidth || 1),
			containerHeight / (contentHeight || 1),
			2
		);
		const z = Math.max(minZoom, fit);
		zoom = z;
		panX = (containerWidth - contentWidth * z) / 2;
		panY = (containerHeight - contentHeight * z) / 2;
	});

	$effect(() => {
		if (!containerWidth || !containerHeight) return;
		const minX = -contentWidth * zoom + visibleMargin;
		const maxX = containerWidth - visibleMargin;
		panX = Math.max(minX, Math.min(maxX, panX));
		const minY = -contentHeight * zoom + visibleMargin;
		const maxY = containerHeight - visibleMargin;
		panY = Math.max(minY, Math.min(maxY, panY));
	});

	function clamp(v: number, lo: number, hi: number): number {
		return Math.max(lo, Math.min(hi, v));
	}

	function sqDist(x1: number, y1: number, x2: number, y2: number): number {
		return (x1 - x2) ** 2 + (y1 - y2) ** 2;
	}

	function sortedTouches(t: TouchList): Touch[] {
		const a: Touch[] = [];
		for (let i = 0; i < t.length; i++) a.push(t[i]!);
		return a.sort((a, b) => a.clientX - b.clientX);
	}

	function isSyntheticMouse(): boolean {
		return touchActive === 0 && performance.now() - lastTouchEnd < 100;
	}

	function anchoredZoom(newZoom: number, ax: number, ay: number) {
		newZoom = clamp(newZoom, minZoom, maxZoom);
		panX = ax - ((ax - panX) / zoom) * newZoom;
		panY = ay - ((ay - panY) / zoom) * newZoom;
		zoom = newZoom;
	}

	function onWheel(e: WheelEvent) {
		if (!focused) return;
		e.preventDefault();
		const r = containerEl!.getBoundingClientRect();
		anchoredZoom(zoom * (1 - e.deltaY * scrollSensitivity), e.clientX - r.left, e.clientY - r.top);
	}

	// https://w3c.github.io/pointerevents/#the-touch-action-css-property
	// touch-action: none is the primary scroll/zoom prevention mechanism;
	// preventDefault on touchmove is defense-in-depth (Chrome may not honor it).
	$effect(() => {
		const el = containerEl;
		if (!el) return;
		el.addEventListener('wheel', onWheel, { passive: false });
		return () => {
			el.removeEventListener('wheel', onWheel);
		};
	});

	function onTouchStart(e: TouchEvent) {
		touchActive = e.touches.length;
		movementSum = 0;

		if (!focused) return;

		const t = sortedTouches(e.touches);

		if (t.length >= 2) {
			pinching = true;
			panning = false;
			pinchZoom = zoom;
			pinchPanX = panX;
			pinchPanY = panY;
			pinchGap = sqDist(t[0].clientX, t[0].clientY, t[1].clientX, t[1].clientY);
			const r = containerEl!.getBoundingClientRect();
			pinchMidX = (t[0].clientX + t[1].clientX) / 2 - r.left;
			pinchMidY = (t[0].clientY + t[1].clientY) / 2 - r.top;
			gestureStartX = (t[0].clientX + t[1].clientX) / 2;
			gestureStartY = (t[0].clientY + t[1].clientY) / 2;
		} else if (t.length === 1) {
			panning = true;
			pinching = false;
			panAnchorX = t[0].clientX - panX;
			panAnchorY = t[0].clientY - panY;
			gestureStartX = t[0].clientX;
			gestureStartY = t[0].clientY;
		}
	}

	function onTouchMove(e: TouchEvent) {
		if (!panning && !pinching) return;

		const t = sortedTouches(e.touches);

		if (pinching && t.length >= 2) {
			const newGap = sqDist(t[0].clientX, t[0].clientY, t[1].clientX, t[1].clientY);
			const newZoom = clamp(pinchZoom * Math.sqrt(newGap / (pinchGap || 1)), minZoom, maxZoom);
			panX = pinchMidX - ((pinchMidX - pinchPanX) / pinchZoom) * newZoom;
			panY = pinchMidY - ((pinchMidY - pinchPanY) / pinchZoom) * newZoom;
			zoom = newZoom;

			const midX = (t[0].clientX + t[1].clientX) / 2;
			const midY = (t[0].clientY + t[1].clientY) / 2;
			movementSum += Math.abs(midX - gestureStartX) + Math.abs(midY - gestureStartY);
		} else if (panning && t.length >= 1) {
			panX = t[0].clientX - panAnchorX;
			panY = t[0].clientY - panAnchorY;
			movementSum +=
				Math.abs(t[0].clientX - gestureStartX) + Math.abs(t[0].clientY - gestureStartY);
		}
	}

	function onTouchEnd(e: TouchEvent) {
		touchActive = Math.max(0, touchActive - e.changedTouches.length);
		if (e.touches.length === 0) {
			panning = false;
			pinching = false;
			lastTouchEnd = performance.now();
		}
	}

	function onTouchCancel() {
		touchActive = 0;
		panning = false;
		pinching = false;
		lastTouchEnd = performance.now();
	}

	function onPointerDown(e: PointerEvent) {
		movementSum = 0;

		if (!focused) return;
		if (e.pointerType === 'touch') return;
		if (isSyntheticMouse()) return;

		panning = true;
		panAnchorX = e.clientX - panX;
		panAnchorY = e.clientY - panY;
		gestureStartX = e.clientX;
		gestureStartY = e.clientY;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent) {
		if (e.pointerType === 'touch') return;
		if (!panning) return;

		panX = e.clientX - panAnchorX;
		panY = e.clientY - panAnchorY;
		movementSum += Math.abs(e.clientX - gestureStartX) + Math.abs(e.clientY - gestureStartY);
	}

	function onPointerUp() {
		panning = false;
	}

	function onPointerCancel() {
		panning = false;
	}

	function onClick() {
		if (movementSum < deadZone) {
			focused = !focused;
		}
	}

	function onKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			focused = !focused;
		}
	}

	let touchAction = $derived(focused ? 'none' : 'auto');
	let cursor = $derived(dragging ? 'grabbing' : focused ? 'grab' : 'auto');
</script>

<!--
	svelte-ignore a11y_no_noninteractive_element_interactions
	The div is made interactive via role="application" + tabindex="-1".
	Svelte's a11y checker does not recognize "application" as an interactive ARIA role
	(only widget-specific roles like button, slider, etc. are recognized).
-->
<div
	bind:this={containerEl}
	class={className}
	style="touch-action: {touchAction}; cursor: {cursor}"
	role="application"
	tabindex="-1"
	ontouchstart={onTouchStart}
	ontouchmove={onTouchMove}
	ontouchend={onTouchEnd}
	ontouchcancel={onTouchCancel}
	onpointerdown={onPointerDown}
	onpointermove={onPointerMove}
	onpointerup={onPointerUp}
	onpointercancel={onPointerCancel}
	onclick={onClick}
	onkeydown={onKeyDown}
>
	{@render children({ zoom, panX, panY, focused, dragging })}
</div>
