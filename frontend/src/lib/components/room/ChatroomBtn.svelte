<script lang="ts">
	import { Trash2, OctagonX } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

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

<div
	class="group flex items-center rounded-sm text-base hover:bg-primary duration-150"
	class:bg-primary={selected}
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
	<button
		class="mr-1 hidden h-6 w-6 shrink-0 p-[2px] group-hover:block"
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
	</button>
</div>
