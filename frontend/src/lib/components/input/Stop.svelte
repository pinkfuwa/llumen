<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import { Square } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<Button class="-mr-2 p-2" {onclick} borderless>
	<Square class="inline-block" />
</Button>
