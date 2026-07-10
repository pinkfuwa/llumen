<script lang="ts">
	import { Trash2, OctagonX } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';
	import { deleteEntry, syncEntry, type MutationStatus } from '$lib/api';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { t } from 'svelte-intl-precompile';

	let { name = $bindable($t('chat.default_title')), id } = $props();

	let selected = $derived(page.params.id === String(id));

	let deleteConfirmed = $state(false);
	let status = $state<MutationStatus>('untried');
	let disabled = $derived(status == 'pending');

	$effect(() => {
		if (name.trim().length == 0) name = $t('chat.default_title');
	});
</script>

<li
	class="group flex cursor-pointer snap-start items-center rounded-sm text-base duration-150 hover:bg-interactive-hover"
	class:bg-interactive-selection={selected}
	class:text-primary={selected}
	onmouseleave={() => {
		deleteConfirmed = false;
	}}
	role="listitem"
>
	{#if selected}
		<form
			class="grow overflow-hidden p-1.5"
			onsubmit={(e) => {
				e.preventDefault();
				let activeElement = document.activeElement;
				if (activeElement) (activeElement as HTMLInputElement).blur();
				status = 'pending';
				syncEntry(id, name).then((x) => {
					status = x;
				});
			}}
		>
			<input class="editor w-full truncate pr-1 group-hover:text-clip" bind:value={name} />
		</form>
	{:else}
		<a class="grow truncate p-1.5 select-none" href="/chat/{encodeURIComponent(id)}">
			{name}
		</a>
	{/if}
	<Button
		class="mr-1 h-6 w-6 shrink-0 p-[0.15rem] text-foreground group-hover:block md:hidden"
		borderless
		{disabled}
		onclick={async () => {
			if (deleteConfirmed) {
				status = 'pending';
				deleteEntry(id).then((x) => {
					if (selected) goto('/chat/new');
					status = x;
				});
			} else {
				deleteConfirmed = true;
			}
		}}
	>
		{#if deleteConfirmed}
			<OctagonX class="h-full w-full" />
		{:else}
			<Trash2 class="h-full w-full" />
		{/if}
	</Button>
</li>
