<script lang="ts">
	import { CheckLine } from '@lucide/svelte';
	import { slide } from 'svelte/transition';

	let { username, onsubmit }: { username: string; onsubmit: (password: string) => void } = $props();

	let password = $state('');
	let passwordCheck = $state('');

	let bounceKey = $state(1);
</script>

<form
	class="mb-4 flex items-center justify-between space-x-2 border-b border-outline pb-2 text-lg"
	onsubmit={() => {
		if (password.length != 0 && password == passwordCheck) {
			onsubmit(password);
		} else {
			bounceKey += 1;
			passwordCheck = '';
		}
	}}
>
	{#key bounceKey}
		<div in:slide={{ duration: 120 }}>
			Type password for <span class="rounded-md bg-hover p-2">{username}</span>
		</div>
	{/key}

	<div class="flex grow flex-col">
		<input
			type="text"
			id="password"
			class="mb-1 rounded-md border border-outline p-1"
			bind:value={password}
			required
		/>
		<input
			type="text"
			id="password"
			class="rounded-md border border-outline p-1"
			bind:value={passwordCheck}
			required
		/>
	</div>
	<button class="mx-1 rounded-md p-1 hover:bg-hover" type="submit"><CheckLine /></button>
</form>
