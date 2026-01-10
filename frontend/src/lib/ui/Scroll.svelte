<script lang="ts">
	import type { Component } from '@lucide/svelte';
	import { untrack } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		damping = 0.2,
		children,
		key = $bindable(null),
		...props
	} = $props<
		HTMLAttributes<HTMLElement> & {
			children: Component;
			damping?: number;
			key?: any;
		}
	>();

	let scrollElement = $state<null | HTMLDivElement>(null);
	let velocity = $state(0);
	let animationFrameId: number | null = null;
	let isAnimating = $derived(animationFrameId != null);

	let lastKey = $state(key);

	$effect(() => {
		if (!scrollElement) return;
		if (key !== lastKey) {
			untrack(() => (lastKey = key));
			scrollElement.scrollTo({
				top: 0,
				behavior: 'instant'
			});
		}
	});

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
</script>

<div
	{...props}
	class="flex overflow-y-auto {props['class'] || ''}"
	style={props.style}
	bind:this={scrollElement}
	{onwheel}
>
	{@render children()}
</div>
