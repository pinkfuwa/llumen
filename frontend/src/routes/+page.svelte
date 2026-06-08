<script lang="ts">
	import { token } from '$lib/rune.svelte';
	import { LoaderCircle } from '@lucide/svelte';
</script>

<div
	class="bg-surface-base flex h-screen flex-col items-center justify-center overflow-hidden text-foreground"
>
	<h1 class="mb-4 text-3xl text-primary/80 md:text-5xl">
		Redirecting...<LoaderCircle
			class="ml-2 inline-block h-12 w-12 animate-spin text-loading-spinner"
		/>
	</h1>
	{#if token.value}
		<a href="/login" class="text-2xl">Go to login page</a>
	{:else}
		<a href="/chat/new" class="text-2xl">Go to chat</a>
	{/if}
</div>
