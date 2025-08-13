<script lang="ts">
	let { files = $bindable([] as Array<{ name: string }>), deletable = false } = $props();

	import { Paperclip, X } from '@lucide/svelte';
</script>

<div class="flex items-center space-x-2">
	{#each files as file, i}
		<div class="group relative rounded-md border border-outline bg-background p-2 hover:bg-hover">
			<Paperclip class="absolute top-2 left-14 h-10 w-10 opacity-20" />
			<div
				class="nobar flex h-10 w-34 items-center justify-center overflow-x-auto break-all select-none"
			>
				{file.name}
			</div>
			{#if deletable}
				<X
					class="absolute top-0 right-0 hidden h-5 w-5 rounded-sm border border-outline bg-background p-[2px] group-hover:block"
					onclick={() => {
						files.splice(i, 1);
						files = files;
					}}
				/>
			{/if}
		</div>
	{/each}
</div>
