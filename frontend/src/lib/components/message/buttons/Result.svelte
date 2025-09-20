<script lang="ts">
	import Warning from '$lib/components/setting/Warning.svelte';
	import Code from '$lib/components/shiki/Code.svelte';
	import Root from '../../markdown/Root.svelte';
	import { OctagonAlert } from '@lucide/svelte';

	let { content } = $props();

	let [error, data] = $derived.by(() => {
		try {
			let data = JSON.parse(content);
			if (typeof data === 'object' && data !== null && 'error' in data) {
				return [(data as { reason?: string }).reason, ''];
			}
			return [undefined, data];
		} catch (e) {
			return [`${e}`, ''];
		}
	});
</script>

{#if error != undefined}
	<div class="my-1">
		<Warning thin>
			<OctagonAlert class="mr-2 inline-block h-7 w-7" />
			{error}
		</Warning>
	</div>
{:else}
	<div class="mt-1 space-y-2">
		<Code lang="json" text={data} />
	</div>
{/if}
