<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	let { name = $bindable('Default chatroom title'), id, selected = false } = $props();

	function emojiName(i: string): string {
		i = i.trim();
		if (!/^\p{Extended_Pictographic}/gu.test(i)) i = '⭕ ' + i;
		return i;
	}
</script>

<li class="group rounded-sm p-1.5 text-base {selected ? 'bg-hover' : 'hover:bg-hover'}">
	{#if selected}
		<div class="flex h-6 items-center">
			<form class="grow overflow-hidden">
				<input class="editor w-full truncate pr-1 group-hover:text-clip" bind:value={name} />
			</form>

			<Trash2 class="hidden h-6 w-6 shrink-0 p-[2px] group-hover:block" />
		</div>
	{:else}
		<a class="flex h-6 items-center select-none" href="/chat/{encodeURIComponent(id)}">
			<div class="grow truncate group-hover:text-clip">
				<span class="h-6">
					{emojiName(name)}
				</span>
			</div>
			<Trash2 class="hidden h-6 w-6 shrink-0 p-[2px] group-hover:block" />
		</a>
	{/if}
</li>
