<script lang="ts">
	import { Trash2, OctagonX } from '@lucide/svelte';

	let {
		name = $bindable('Default chatroom title'),
		id,
		selected = false,
		ondelete = () => {}
	} = $props();

	let checked = $state(false);
</script>

<div
	class="group flex rounded-sm text-base {selected ? 'bg-hover' : 'hover:bg-hover'} items-center"
	onmouseleave={() => {
		checked = false;
	}}
	role="listitem"
>
	{#if selected}
		<form class="grow overflow-hidden p-1.5">
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
