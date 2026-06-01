<script lang="ts">
	let {
		files = $bindable([] as Array<{ name: string; type?: string; id?: number }>),
		deletable = false,
		mimes = []
	}: {
		files: Array<{ name: string; type?: string; id?: number }>;
		deletable?: boolean;
		mimes?: string[];
	} = $props();

	import { ArrowDownToLine, X, AlertTriangle } from '@lucide/svelte';
	import { download } from '$lib/api/files.svelte';
	import { isMimeSupported } from '../input/fileTypes';
	import InteractiveRow from '$lib/ui/InteractiveRow.svelte';

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
		<InteractiveRow
			class="group flex min-h-10 flex-row rounded-md border border-border bg-popover p-3"
		>
			<InteractiveRow
				class="my-auto mr-2 shrink-0 rounded-md p-1 focus:ring-4 focus:ring-ring focus:outline-none"
			>
				{#if deletable}
					{#if file.type && isMimeSupported(file.type, mimes)}
						<X
							class="h-7 w-7"
							onclick={() => {
								files.splice(i, 1);
								files = files;
							}}
						/>
					{:else}
						<AlertTriangle
							class="h-7 w-7"
							onclick={() => {
								files.splice(i, 1);
								files = files;
							}}
						/>
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
			</InteractiveRow>
			<div class="flex min-w-0 grow items-center justify-center">
				<div class="overflow-x-auto">
					{file.name}
				</div>
			</div>
		</InteractiveRow>
	{/each}
</div>
