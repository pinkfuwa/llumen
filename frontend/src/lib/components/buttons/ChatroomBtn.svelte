<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	let { name = $bindable('Default chatroom title'), id, selected = false } = $props();
</script>

<li
	class="group text-md flex items-center justify-between rounded-sm p-1.5 {selected
		? 'bg-hover'
		: 'hover:bg-hover'}"
>
	<a href="/chat/{encodeURIComponent(id)}">
		{#if selected}
			<form
				class="grow items-center"
				onsubmit={(e) => {
					e.preventDefault();
				}}
			>
				<input class="editor h-6 w-full truncate" bind:value={name} />
			</form>
		{:else}
			<div class="h-6 w-full grow items-center truncate select-none">
				{name}
			</div>
			<Trash2 class="hidden p-[2px] group-hover:block" />
		{/if}
		<Trash2 class="hidden p-[2px] group-hover:block" />
	</a>
</li>
