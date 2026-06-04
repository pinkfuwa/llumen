<script lang="ts">
	import { ToolCase } from '@lucide/svelte';
	import { Accordion } from 'bits-ui';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { slide } from 'svelte/transition';
	import Badge from '$lib/ui/Badge.svelte';

	let { children, toolname = 'Default Tool' } = $props();

	let displayName = $derived(toolname === 'Default Tool' ? m['chat.default_tool'](lang) : toolname);

	let open = $state(false);

	const triggerStyle =
		'flex flex-row flex-nowrap items-center rounded p-2 cursor-pointer duration-150 hover:bg-interactive-hover';
</script>

<Accordion.Root type="multiple">
	<Accordion.Item>
		<Accordion.Header>
			<Accordion.Trigger class={triggerStyle}>
				<ToolCase class="mr-2" />
				{#if !open}
					<span class="mr-1"> {m['chat.calling'](lang)} </span>
				{/if}
				<Badge>
					{displayName}
				</Badge>
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
