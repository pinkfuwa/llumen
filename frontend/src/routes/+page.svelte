<script lang="ts">
	import { goto } from '$app/navigation';
	import { token } from '$lib/store';
	import { LoaderCircle } from '@lucide/svelte';

	$effect(() => {
		if ($token) goto('/chat/new');
		else goto('/login');
	});
</script>

<div class="flex h-screen flex-col items-center justify-center bg-login-bg">
	<h1 class="mb-4 text-3xl md:text-5xl">
		Redirecting...<LoaderCircle class="ml-2 inline-block h-12 w-12 animate-spin" />
	</h1>
	{#if $token}
		<a href="/login" class="text-2xl">Go to login page</a>
	{:else}
		<a href="/chat/new" class="text-2xl">Go to chat</a>
	{/if}
</div>
