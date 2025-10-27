<script lang="ts">
	import { RawAPIFetch } from '$lib/api/state/errorHandle';

	let { files = $bindable([] as Array<{ name: string; id?: number }>), deletable = false } =
		$props();

	let anchor = $state<HTMLAnchorElement | null>(null);
	import { Paperclip, X } from '@lucide/svelte';
</script>

<div>
	<div class="flex items-center space-x-2">
		{#each files as file, i}
			<div class="group bg-background hover:bg-hover relative rounded-md border border-outline p-2">
				<Paperclip class="absolute top-2 left-14 h-10 w-10 opacity-20" />
				<div
					class="nobar flex h-10 w-34 items-center justify-center overflow-x-auto break-all select-none"
				>
					{file.name}
				</div>
				{#if deletable}
					<X
						class="bg-background absolute top-0 right-0 hidden h-5 w-5 rounded-sm border border-outline p-[0.15rem] group-hover:block"
						onclick={() => {
							files.splice(i, 1);
							files = files;
						}}
					/>
				{/if}
			</div>
		{/each}
	</div>
</div>
