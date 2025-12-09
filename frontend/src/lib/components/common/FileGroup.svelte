<script lang="ts">
	let { files = $bindable([] as Array<{ name: string; id?: number }>), deletable = false } =
		$props();

	import { ArrowDownToLine, X } from '@lucide/svelte';
	import { download } from '$lib/api/files';

	async function downloadFile(fileId: number, fileName: string) {
		let url = await download(fileId);
		if (url != undefined) {
			const link = document.createElement('a');
			link.href = url;
			link.download = fileName;
			document.body.appendChild(link);
			link.click();
			document.body.removeChild(link);
			window.URL.revokeObjectURL(url);
		}
	}
</script>

<div class="space-y-2">
	{#each files as file, i}
		<div
			class="group bg-background hover:bg-hover flex min-h-10 flex-row rounded-md border border-outline p-3"
		>
			<div
				class="my-auto mr-2 shrink-0 rounded-md p-1 duration-150 hover:bg-primary hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none"
			>
				{#if deletable}
					<X
						class="h-7 w-7"
						onclick={() => {
							files.splice(i, 1);
							files = files;
						}}
					/>
				{:else}
					<ArrowDownToLine
						class="h-7 w-7"
						onclick={() => {
							if (file.id) {
								downloadFile(file.id, file.name);
							}
						}}
					/>
				{/if}
			</div>
			<div class="flex min-w-0 grow items-center justify-center overflow-x-auto">
				{file.name}
			</div>
		</div>
	{/each}
</div>
