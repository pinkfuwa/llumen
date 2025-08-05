<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	let { name = $bindable('Default chatroom title'), id, selected = false } = $props();
</script>

<li
	class="group text-md flex items-center justify-between rounded-sm p-1.5 {selected
		? 'bg-hover'
		: 'hover:bg-hover'}"
>
	{#if selected}
		<form class="grow items-center">
			<input class="editor h-6 w-full truncate" bind:value={name} />
		</form>

		<Trash2 class="hidden p-[2px] group-hover:block" />
	{:else}
		<a class="flex grow items-center justify-between" href="/chat/{encodeURIComponent(id)}">
			<div class="h-6 w-full grow items-center truncate select-none">
				{name}
			</div>
			<Trash2 class="hidden p-[2px] group-hover:block" />
		</a>
	{/if}
</li>
