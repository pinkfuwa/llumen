<script lang="ts">
	import { ArrowLeft, Bot, Plus } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';

	import { open } from './open.svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';

	let addition = $derived(page.params.id != undefined);
</script>

<div class="mb-4 flex shrink-0 items-center justify-between border-b border-border pb-1">
	<div class="flex items-center text-lg font-semibold select-none">
		<Bot class="mx-2 h-6 w-6" />
		<a href="https://github.com/pinkfuwa/llumen" target="_blank">llumen</a>
	</div>
	<Button
		class="rounded-lg p-3"
		onclick={() => {
			open.val = !open.val;
		}}
	>
		<ArrowLeft />
	</Button>
</div>
<div
	data-state={addition ? 'show' : 'hide'}
	class="shrink-0 grow overflow-hidden transition-[max-height] duration-300 ease-out data-[state=hide]:max-h-0 data-[state=show]:max-h-32"
>
	<Button
		href="/chat/new"
		class="flex w-full items-center justify-center rounded-md p-1.5 font-semibold"
	>
		<Plus class="mr-2 h-5 w-5" />
		{m['chat.new']()}
	</Button>
</div>
