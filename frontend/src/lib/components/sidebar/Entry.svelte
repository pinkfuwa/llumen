<script lang="ts">
	import { Trash2, OctagonX } from '@lucide/svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import Button from '$lib/ui/Button.svelte';
	import { deleteEntry, syncEntry } from '$lib/api';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';

	let { name = $bindable(m['chat.default_title'](lang)), id } = $props();

	let selected = $derived(page.params.id === String(id));
	// onupdate={(newName: string) => syncEntry(chatroom.id, newName)}

	let deleteConfirmed = $state(false);
	let disabled = $state(false);

	$effect(() => {
		if (name.trim().length == 0) name = m['chat.default_title'](lang);
	});

	async function runBusy<T>(p: Promise<T>) {
		disabled = true;
		await p;
		if (selected) goto('/chat/new');
		disabled = false;
	}

	async function handleUpdate(e: Event) {
		e.preventDefault();
		runBusy(syncEntry(id, name));
		(e.target! as HTMLInputElement).blur();
	}
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
		<form class="grow overflow-hidden p-1.5" onsubmit={handleUpdate}>
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
		onclick={() => {
			if (deleteConfirmed) runBusy(deleteEntry(id));
			else deleteConfirmed = true;
		}}
	>
		{#if deleteConfirmed}
			<OctagonX class="h-full w-full" />
		{:else}
			<Trash2 class="h-full w-full" />
		{/if}
	</Button>
</li>
