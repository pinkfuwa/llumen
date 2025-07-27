<script lang="ts">
	import { CreateUser } from '$lib/api/user';
	import { CheckLine } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { fade } from 'svelte/transition';
	import { useToken } from '$lib/store';
	let { params } = $props();

	let token = useToken();
	let username = params.username;
	let password = $state('');

	const createUserMutation = CreateUser();

	function handleSubmit(event: Event) {
		event.preventDefault();
		$createUserMutation.mutate(
			{
				username: username,
				password: password,
				token: token.current!
			},
			{
				onSuccess: () => {
					goto('/setting/admin');
				}
			}
		);
	}
</script>

{#if $createUserMutation.failureCount > 0}
	<div class="mb-2 rounded-lg bg-red-700 hover:bg-red-500" in:fade={{ duration: 180 }}>
		<div class="ml-2 bg-background p-3 font-semibold hover:bg-hover">
			User creation failed: {$createUserMutation.failureReason}
		</div>
	</div>
{/if}

<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<div>Type password for <span class="rounded-md bg-hover p-2">{username}</span></div>
	<form class="flex items-center justify-between" onsubmit={handleSubmit}>
		<input
			type="text"
			id="password"
			class="rounded-md border border-outline p-1"
			bind:value={password}
		/>
		<button class="mx-1 rounded-md p-1 hover:bg-hover" type="submit"><CheckLine /></button>
	</form>
</div>
