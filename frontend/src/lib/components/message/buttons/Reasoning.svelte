<script lang="ts">
	import { Brain } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { Accordion } from 'bits-ui';
	import Root from '$lib/components/markdown/Root.svelte';
	const { content }: { content: string } = $props();

	let showReasoning = $state(false);

	let lines = $derived(content.split('\n'));
</script>

<Accordion.Root type="multiple">
	<Accordion.Item>
		<Accordion.Header>
			<Accordion.Trigger
				class="flex flex-row flex-nowrap rounded p-2 duration-150 hover:bg-primary hover:text-text-hover"
			>
				<Brain class="mr-2" />
				<span>
					{$_('chat.reasoning')}
				</span>
			</Accordion.Trigger>
		</Accordion.Header>
		<Accordion.Content
			class="
				py-2
				slide-out-to-start-2
				fade-in
				fade-out
				slide-in-from-top-2
				data-[state=close]:animate-out
				data-[state=open]:animate-in
			"
		>
			{#each lines as line}
				<p class="whitespace-pre-wrap">{line}</p>
			{/each}
		</Accordion.Content>
	</Accordion.Item>
</Accordion.Root>
