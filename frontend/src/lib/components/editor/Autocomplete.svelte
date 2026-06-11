<script lang="ts">
	import type { CompletionOption } from './completion';

	let {
		options = [] as CompletionOption[],
		x = 0,
		y = 0,
		onselect = undefined as ((opt: CompletionOption) => void) | undefined,
		onclose = undefined as (() => void) | undefined
	} = $props();

	let selectedIndex = $state(0);

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return {
			destroy() {
				node.remove();
			}
		};
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = (selectedIndex + 1) % options.length;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = (selectedIndex - 1 + options.length) % options.length;
		} else if (e.key === 'Tab') {
			e.preventDefault();
			onselect?.(options[selectedIndex]);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			onclose?.();
		}
	}

	$effect(() => {
		function handleClickOutside(e: MouseEvent) {
			const target = e.target as HTMLElement;
			if (!target.closest('[data-autocomplete]')) {
				onclose?.();
			}
		}
		document.addEventListener('mousedown', handleClickOutside);
		return () => document.removeEventListener('mousedown', handleClickOutside);
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div
	use:portal
	data-autocomplete
	class="fixed z-100 hidden min-w-[12rem] rounded-lg border border-border bg-popover p-1 font-mono text-sm shadow-lg lg:block"
	style="left:{x}px; top:{y}px"
>
	{#each options as opt, i}
		<button
			class="flex w-full cursor-pointer items-center rounded-md bg-accent px-2 py-1.5 text-left text-accent-foreground transition-colors"
			class:bg-interactive-hover={i === selectedIndex}
			onclick={() => onselect?.(opt)}
			onmouseenter={() => (selectedIndex = i)}
		>
			{opt.label}
		</button>
	{/each}
</div>
