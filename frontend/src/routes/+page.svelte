<script lang="ts">
	import { goto } from '$app/navigation';
	import { useToken } from '$lib/store';

	let token = useToken();

	$effect(() => {
		if ($token != '') goto('/chat/new');
		else goto('/login');
	});
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<h1 class="mb-4 text-3xl font-light md:text-5xl">Redirecting...</h1>
	{#if $token == ''}
		<a href="/login" class="text-2xl">Go to login page</a>
	{:else}
		<a href="/chat/new" class="text-2xl">Go to chat</a>
	{/if}
</div>
