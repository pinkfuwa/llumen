<script lang="ts">
	import { Brain } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { Collapsible } from 'bits-ui';
	let { content, open = $bindable(false) }: { content: string; open?: boolean } = $props();

	const lines = $derived(content.split('\n'));
</script>

<Collapsible.Root bind:open>
	<Collapsible.Trigger
		class="flex flex-row flex-nowrap rounded p-2 duration-150 hover:bg-primary hover:text-text-hover"
	>
		<Brain class="mr-2" />
		<span>
			{$_('chat.reasoning')}
		</span>
	</Collapsible.Trigger>
	<Collapsible.Content
		class="py-2 slide-out-to-start-2 fade-in fade-out slide-in-from-top-2 data-[state=close]:animate-out data-[state=open]:animate-in"
	>
		{#each lines as line}
			<p class="whitespace-pre-wrap">{line}</p>
		{/each}
	</Collapsible.Content>
</Collapsible.Root>
