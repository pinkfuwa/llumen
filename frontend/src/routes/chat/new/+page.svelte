<script lang="ts">
	import { fade } from 'svelte/transition';
	import { MessageInput, Copyright } from '$lib/components';
	import { t } from 'svelte-intl-precompile';
</script>

<svelte:head>
	<title>{$t('chat.title')}</title>
</svelte:head>

<main class="flex h-full w-full flex-col justify-center">
	<h1
		class="mx-auto mb-4 bg-gradient-to-r from-primary to-primary/50 bg-clip-text pb-4 text-[11vw] font-semibold text-transparent select-none md:text-[max(4rem,5vw)]"
		in:fade={{ duration: 150 }}
	>
		{$t('chat.welcome')}
	</h1>
	<MessageInput large />
</main>

<Copyright />
