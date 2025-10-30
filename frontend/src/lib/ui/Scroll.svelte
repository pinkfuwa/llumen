<script lang="ts">
	import type { Component } from '@lucide/svelte';
	import { setContext, untrack } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { get, writable } from 'svelte/store';

	const props = $props<
		HTMLAttributes<HTMLElement> & {
			children: Component;
			damping?: number;
		}
	>();

	const { damping = 0.2, children } = props;

	let scrollElement = $state<null | HTMLDivElement>(null);
	let velocity = $state(0);
	let animationFrameId: number | null = null;
	let isAnimating = $derived(animationFrameId != null);

	const velocityThreshold = 3;

	function onwheel(event: WheelEvent) {
		if (scrollElement == null || scrollElement.getBoundingClientRect().right - 220 > event.pageX)
			return;
		const activeTage = document.activeElement?.tagName;
		if (activeTage == 'TEXTAREA' || activeTage == 'INPUT') return;

		if (Math.abs(event.deltaY / event.deltaX) < 1) return;
		event.preventDefault();

		if (animationFrameId !== null) {
			cancelAnimationFrame(animationFrameId);
			animationFrameId = null;
		}

		const impulse = event.deltaY * (1 - damping);
		velocity += impulse;

		if (!isAnimating) animate();
	}

	function animate() {
		if (!scrollElement) {
			if (animationFrameId !== null) {
				cancelAnimationFrame(animationFrameId);
				animationFrameId = null;
			}
			return;
		}

		velocity *= 1 - damping;

		scrollElement.scrollTop += Math.round(velocity);

		if (Math.abs(velocity) > velocityThreshold) {
			animationFrameId = requestAnimationFrame(animate);
		} else {
			isAnimating = false;
			velocity = 0;
			animationFrameId = null;
		}
	}

	let scrollTop = $state(0);
	let scrollHeight = $state(0);

	let lock = writable(false);
	setContext('scrollLock', lock);

	$effect(() =>
		lock.subscribe((value) => {
			if (value || !scrollElement) return;
			let scrollTopValue = untrack(() => scrollTop);

			if (scrollTopValue < 0)
				scrollTopValue = Math.min(scrollTopValue - scrollElement.scrollHeight + scrollHeight, 0);

			scrollTop = scrollTopValue;
			scrollElement.scrollTo({
				top: scrollTopValue,
				behavior: 'instant'
			});
		})
	);

	$effect(() => {
		if (scrollElement) scrollHeight = scrollElement.scrollHeight;
	});
</script>

<div
	{...props}
	class="flex overflow-y-auto {props['class'] || ''}"
	style={props.style}
	bind:this={scrollElement}
	{onwheel}
	onscroll={() => {
		if (!scrollElement || get(lock)) return;
		scrollTop = scrollElement.scrollTop;
		scrollHeight = scrollElement.scrollHeight;
		console.log('update top to', scrollElement.scrollTop);
	}}
>
	{@render children()}
</div>
