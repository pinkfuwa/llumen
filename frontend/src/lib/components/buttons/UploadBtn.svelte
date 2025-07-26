<script lang="ts">
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Upload } from '@lucide/svelte';
</script>

<button class="rounded-md bg-background p-1 hover:bg-hover">
	<Tooltip content="Upload file">
		<Upload class="inline-block" />
	</Tooltip>
</button>
