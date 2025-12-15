<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import { CornerDownLeft } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement>, disabled = false } = $props();
</script>

<Button class="-mr-2 p-2" {disabled} {onclick} borderless aria-label="send">
	<CornerDownLeft class="inline-block" />
</Button>
