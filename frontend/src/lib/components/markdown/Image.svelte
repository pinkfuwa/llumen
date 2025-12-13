<script lang="ts">
	import type { ASTNode } from './lexer/parser';
	import { BookX } from '@lucide/svelte';

	let { node }: { node: ASTNode } = $props();

	const src = $derived(node.children.find((x) => x.type == 'URL')?.text);

	let errored = $state(false);
	let data = $derived(errored || src == undefined ? 'error' : 'normal');

	function uselessFn(data: any) {}
	$effect(() => {
		uselessFn(node);
		errored = false;
	});
</script>

<div class="inline-block align-middle">
	{#if src != undefined}
		<img
			{src}
			alt=""
			style="max-width: 100%;"
			onerror={() => (errored = true)}
			class="max-h-[70vh] data-[state=error]:hidden"
			data-state={data}
		/>
	{/if}
	<div
		class="flex h-50 w-60 flex-col items-center justify-center rounded-md border border-outline text-lg data-[state=normal]:hidden"
		data-state={data}
	>
		<BookX class="h-10 w-10" />
		<span class="mt-1">Image not found</span>
	</div>
</div>
