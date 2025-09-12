<script lang="ts">
	import Warning from '$lib/components/setting/Warning.svelte';
	import Root from '../../markdown/Root.svelte';
	import { OctagonAlert } from '@lucide/svelte';

	let { content } = $props();

	let error = $derived.by(() => {
		try {
			let data = JSON.parse(content);
			if (typeof data === 'object' && data !== null && 'error' in data) {
				return data as { error: string; reason?: string };
			}
		} catch (e) {}
	});
</script>

{#if error != undefined}
	<div class="my-1">
		<Warning thin>
			<OctagonAlert class="mr-2 inline-block h-7 w-7" />
			{error.reason}
		</Warning>
	</div>
{:else}
	<div class="mt-1 space-y-2">
		<Root source={content} />
	</div>
{/if}
