<script lang="ts">
	import type { MouseEventHandler } from 'svelte/elements';

	let {
		rotationIntensity = 10,
		shadowIntensity = 0.8,
		children,
		type = undefined,
		disabled = null,
		class: className = '',
		onclick = undefined as MouseEventHandler<HTMLButtonElement> | null | undefined
	} = $props();

	function handleMouseMove(event: MouseEvent) {
		const target = event.currentTarget! as HTMLElement;

		const rect = target.getBoundingClientRect();
		const x = event.clientX - rect.left;
		const y = event.clientY - rect.top;
		const centerX = rect.width / 2;
		const centerY = rect.height / 2;

		const rotateX = ((y - centerY) / centerY) * rotationIntensity;
		const rotateY = ((x - centerX) / centerX) * -rotationIntensity;

		const shadowX = -rotateY * shadowIntensity;
		const shadowY = rotateX * shadowIntensity;

		target.style.transform = `rotateX(${rotateX}deg) rotateY(${rotateY}deg)`;
		target.style.boxShadow = `${shadowX}px ${shadowY}px 36px rgba(0,20,60,0.12)`;
	}

	function handleMouseLeave(event: MouseEvent) {
		const target = event.currentTarget! as HTMLElement;

		target.style.transform = `rotateX(0deg) rotateY(0deg)`;
		target.style.boxShadow = `0px 0px 36px rgba(0,20,60,0.12)`;
	}
</script>

<button
	class={className
		.split(' ')
		.concat(['transition-all', 'ease-out'])
		.filter((x) => x != '')
		.join(' ')}
	onmousemove={handleMouseMove}
	onmouseleave={handleMouseLeave}
	{type}
	{disabled}
	{onclick}
>
	{@render children()}
</button>
