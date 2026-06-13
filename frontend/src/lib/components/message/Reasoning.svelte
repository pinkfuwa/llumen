<script lang="ts">
	import { Brain } from '@lucide/svelte';
	import { Collapsible } from 'bits-ui';
	import { t } from 'svelte-intl-precompile';
	let { content, open = $bindable(false) }: { content: string; open?: boolean } = $props();

	// svelte-ignore state_referenced_locally
	let lines = $state(content.split('\n'));
	// svelte-ignore state_referenced_locally
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
		class="flex cursor-pointer flex-row flex-nowrap rounded p-2 duration-150 hover:bg-interactive-hover"
	>
		<Brain class="mr-2" />
		<span>
			{$t('chat.reasoning')}
		</span>
	</Collapsible.Trigger>
	<Collapsible.Content
		class="py-2 whitespace-pre-wrap slide-out-to-start-2 fade-in fade-out slide-in-from-top-2 data-[state=close]:animate-out data-[state=open]:animate-in"
	>
		{content}
	</Collapsible.Content>
</Collapsible.Root>
