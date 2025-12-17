<script lang="ts">
	import { ToolCase } from '@lucide/svelte';
	import { Accordion } from 'bits-ui';
	import { _ } from 'svelte-i18n';
	import { slide } from 'svelte/transition';

	let { children, toolname = 'Default Tool' } = $props();

	let displayName = $derived(toolname === 'Default Tool' ? $_('chat.default_tool') : toolname);

	let open = $state(false);
</script>

<Accordion.Root type="multiple">
	<Accordion.Item>
		<Accordion.Header>
			<Accordion.Trigger
				class="flex flex-row flex-nowrap items-center rounded p-2 duration-150 hover:bg-primary hover:text-text-hover"
			>
				<ToolCase class="mr-2" />
				{#if !open}
					<span class="mr-1"> {$_('chat.calling')} </span>
				{/if}
				<span class="rounded-md bg-primary px-2 py-[2px] text-text-hover">
					{displayName}
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
			{@render children()}
		</Accordion.Content>
	</Accordion.Item>
</Accordion.Root>
