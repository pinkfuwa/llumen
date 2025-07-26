<script lang="ts">
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Upload } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<button class="rounded-md bg-background p-1 hover:bg-hover" {onclick}>
	<Tooltip content="Upload file">
		<Upload class="inline-block" />
	</Tooltip>
</button>
