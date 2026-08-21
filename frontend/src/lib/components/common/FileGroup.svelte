<script lang="ts">
	let {
		files = $bindable([] as Array<{ name: string; type?: string; id?: number }>),
		deletable = false,
		allowedUnsupported = [] as Array<{ name: string }>
	}: {
		files: Array<{ name: string; type?: string; id?: number }>;
		deletable?: boolean;
		allowedUnsupported?: Array<{ name: string }>;
	} = $props();

	import { ArrowDownToLine, X, AlertTriangle } from '@lucide/svelte';
	import { download } from '$lib/api/files.svelte';

	function isAllowedUnsupported(file: { name: string }) {
		return allowedUnsupported.some((candidate) => candidate.name === file.name);
	}

	function removeFile(index: number) {
		files.splice(index, 1);
		files = files;
	}

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
			class="group flex min-h-10 cursor-pointer flex-row rounded-md border border-border bg-popover p-3 duration-150 hover:bg-interactive-hover"
		>
			<div
				class="my-auto mr-2 shrink-0 cursor-pointer rounded-md p-1 duration-150 hover:bg-interactive-hover focus:ring-4 focus:ring-ring focus:outline-none"
			>
				{#if deletable}
					{#if isAllowedUnsupported(file)}
						<AlertTriangle class="h-7 w-7 group-hover:hidden" onclick={() => removeFile(i)} />
						<X class="hidden h-7 w-7 group-hover:block" onclick={() => removeFile(i)} />
					{:else}
						<X class="h-7 w-7" onclick={() => removeFile(i)} />
					{/if}
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
			<div class="flex min-w-0 grow items-center justify-center">
				<div class="overflow-x-auto">
					{file.name}
				</div>
			</div>
		</div>
	{/each}
</div>
