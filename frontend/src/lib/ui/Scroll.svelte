<script lang="ts">
	import type { Component } from '@lucide/svelte';
	import type { HTMLAttributes } from 'svelte/elements';

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
	class="overflow-y-auto {props['class'] || ''}"
	style={props.style}
	bind:this={scrollElement}
	{onwheel}
>
	{@render children()}
</div>
