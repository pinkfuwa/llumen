<script lang="ts">
	import { Trash2, OctagonX } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import InteractiveRow from '$lib/ui/InteractiveRow.svelte';
	import Button from '$lib/ui/Button.svelte';

	let {
		name = $bindable($_('chat.default_title')),
		id,
		selected = false,
		ondelete = () => {},
		onupdate = ((newName: string) => {}) as (newName: string) => void
	} = $props();

	let checked = $state(false);

	$effect(() => {
		if (name.trim().length == 0) name = $_('chat.default_title');
	});
</script>

<svelte:head>
	{#if selected && name != ''}
		<title>{name}</title>
	{/if}
</svelte:head>

<InteractiveRow
	class="group flex items-center rounded-sm text-base"
	{selected}
	onmouseleave={() => {
		checked = false;
	}}
	role="listitem"
>
	{#if selected}
		<form
			class="grow overflow-hidden p-1.5"
			onsubmit={() => {
				onupdate(name);
			}}
		>
			<input class="editor w-full truncate pr-1 group-hover:text-clip" bind:value={name} />
		</form>
	{:else}
		<a class="grow truncate p-1.5 select-none" href="/chat/{encodeURIComponent(id)}">
			{name}
		</a>
	{/if}
	<Button
		class="mr-1 h-6 w-6 shrink-0 p-[0.15rem] text-foreground group-hover:block md:hidden"
		borderless
		onclick={() => {
			if (!checked) checked = true;
			else ondelete();
		}}
	>
		{#if checked}
			<OctagonX class="h-full w-full" />
		{:else}
			<Trash2 class="h-full w-full" />
		{/if}
	</Button>
</InteractiveRow>
