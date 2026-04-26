<script lang="ts">
	import { Brain } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { Collapsible } from 'bits-ui';
	let { content, open = $bindable(false) }: { content: string; open?: boolean } = $props();

	let lines = $state(content.split('\n'));
	let prefix = $state(content);

	$effect(() => {
		if (content.startsWith(prefix)) {
			let newLines = content.split('\n');
			lines[lines.length - 1] = newLines[lines.length - 1];
			for (let i = lines.length; i < newLines.length; i++) {
				lines.push(newLines[i]);
			}
			prefix = content;
		} else {
			lines = content.split('\n');
			prefix = content;
		}
	});
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
