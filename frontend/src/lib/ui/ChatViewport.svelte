<script lang="ts">
	import type { Snippet } from 'svelte';
	let {
		element = $bindable(),
		widen = false,
		children,
		composer
	}: {
		element?: HTMLDivElement;
		widen?: boolean;
		children: Snippet;
		composer: Snippet;
	} = $props();
</script>

<div
	class="nobar anchor-none relative flex h-full flex-col overflow-y-auto transition-all lg:data-widen:px-24"
	bind:this={element}
	data-widen={widen ? '' : undefined}
>
	<!-- Flexible top spacing shrinks when reasoning opens and pulls earlier content upward. -->
	<div class="min-h-16 shrink-0 snap-start"></div>
	<div class="shrink-0">{@render children()}</div>
	<div class="min-h-4 shrink-0 grow"></div>
	<div class="sticky bottom-1 z-10 flex shrink-0 snap-end justify-center">
		{@render composer()}
	</div>
</div>
