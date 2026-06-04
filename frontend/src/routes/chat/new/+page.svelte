<script lang="ts">
	import { fade } from 'svelte/transition';
	import { MessageInput, Copyright } from '$lib/components';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { page } from '$app/state';
	import { inputContent } from '$lib/components/input/state.svelte';

	$effect(() => {
		const param = page.url.searchParams;
		if (param.has('q')) inputContent.val = param.get('q')!;
	});
</script>

<svelte:head>
	<title>{m['chat.title'](lang)}</title>
</svelte:head>

<main class="flex h-full w-full flex-col justify-center">
	<h1
		class="mx-auto mb-4 bg-gradient-to-r from-primary to-primary/50 bg-clip-text pb-4 text-[11vw] font-semibold text-transparent select-none md:text-[max(4rem,5vw)]"
		in:fade={{ duration: 150 }}
	>
		{m['chat.welcome'](lang)}
	</h1>
	<MessageInput large />
</main>

<Copyright />
