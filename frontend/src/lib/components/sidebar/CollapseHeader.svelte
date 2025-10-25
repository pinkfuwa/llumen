<script lang="ts">
	import { ArrowLeft, Bot } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import Button from '$lib/ui/Button.svelte';

	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<div class="flex items-center justify-between">
	<div class="flex items-center text-lg font-semibold select-none">
		<Bot class="mx-2 h-6 w-6" />
		<a href="https://github.com/pinkfuwa/llumen" target="_blank">llumen</a>
	</div>
	<Button class="rounded-lg p-3" {onclick}>
		<ArrowLeft />
	</Button>
</div>
