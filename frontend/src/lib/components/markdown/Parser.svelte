<script lang="ts">
	import Parser from './Parser.svelte';
	import { renderers } from './renderers';

	let {
		type = undefined as keyof typeof renderers | undefined,
		tokens = undefined,
		header = undefined,
		rows = undefined,
		ordered = false,
		monochrome = false,
		...rest
	} = $props();
</script>

{#if !type}
	{#each tokens as token}
		<Parser {...token} {renderers} {monochrome} />
	{/each}
{:else if renderers[type]}
	{#if type === 'table'}
		<renderers.table>
			<renderers.tablehead>
				<renderers.tablerow>
					{#each header as headerItem, i}
						<renderers.tablecell header={true} align={rest.align[i] || 'center'}>
							<Parser tokens={headerItem.tokens} {renderers} />
						</renderers.tablecell>
					{/each}
				</renderers.tablerow>
			</renderers.tablehead>
			<renderers.tablebody>
				{#each rows as row}
					<renderers.tablerow>
						{#each row as cells, i}
							<renderers.tablecell header={false} align={rest.align[i] || 'center'}>
								<Parser tokens={cells.tokens} {renderers} />
							</renderers.tablecell>
						{/each}
					</renderers.tablerow>
				{/each}
			</renderers.tablebody>
		</renderers.table>
	{:else if type === 'list'}
		{#if ordered}
			<renderers.list {ordered} start={rest.start} {...rest}>
				{#each rest.items as item}
					{@const SvelteComponent = renderers.orderedlistitem || renderers.listitem}
					<SvelteComponent {...item}>
						<Parser tokens={item.tokens} {renderers} />
					</SvelteComponent>
				{/each}
			</renderers.list>
		{:else}
			<renderers.list {ordered} start={rest.start} {...rest}>
				{#each rest.items as item}
					{@const SvelteComponent_1 = renderers.unorderedlistitem || renderers.listitem}
					<SvelteComponent_1 {...item}>
						<Parser tokens={item.tokens} {renderers} />
					</SvelteComponent_1>
				{/each}
			</renderers.list>
		{/if}
	{:else}
		<!-- we don't care the type -->
		{@const Render = renderers[type] as any}
		<Render {...rest} {monochrome}>
			{#if tokens}
				<Parser {tokens} {renderers} {monochrome} />
			{:else}
				{rest.raw}
			{/if}
		</Render>
	{/if}
{/if}
