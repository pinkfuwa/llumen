<script lang="ts">
	import Root from '$lib/markdown/Root.svelte';

	let { content } = $props();
</script>

<div class="mt-1 space-y-2">
	<Root source={content} />
</div>
