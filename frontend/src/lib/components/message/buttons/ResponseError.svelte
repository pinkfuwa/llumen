<script lang="ts">
	import Warning from '$lib/components/setting/Warning.svelte';
	import { CloudAlert } from '@lucide/svelte';
	let { content } = $props();
</script>

<Warning>
	<CloudAlert class="mr-2 inline-block h-7 w-7" />
	{content}
</Warning>
