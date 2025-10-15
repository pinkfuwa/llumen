<script lang="ts">
	import type { Component } from '@lucide/svelte';
	import type { HTMLAttributes } from 'svelte/elements';

	const props = $props<
		HTMLAttributes<HTMLElement> & {
			children: Component;
			speed?: number;
			friction?: number;
		}
	>();

	const { speed = 0.15, friction = 0.92, children } = props;

	let scrollElement = $state<null | HTMLDivElement>(null);
	let velocity = $state(0);
	let staticFration = $state(0);
	let isAnimating = $derived(Math.abs(velocity) > Math.max(1, staticFration));

	function onwheel(event: WheelEvent) {
		event.preventDefault();

		const sameDirection = event.deltaY * velocity > 0;
		const delta = event.deltaY;
		const speedDelta = delta * speed + 1;

		if (isAnimating && sameDirection) {
			velocity *= 1.7;
			if (Math.abs(velocity) < Math.abs(speedDelta)) velocity = speedDelta;
		} else {
			staticFration = Math.min(14, delta * 0.1);
			velocity = speedDelta;
			animate();
		}
	}

	function animate() {
		if (!scrollElement || !isAnimating) velocity = 0;
		else {
			scrollElement.scrollTop += Math.round(velocity);
			velocity *= friction;
			requestAnimationFrame(animate);
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
