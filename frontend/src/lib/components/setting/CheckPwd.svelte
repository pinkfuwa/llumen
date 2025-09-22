<script lang="ts">
	import { CheckLine, X } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import Warning from './Warning.svelte';

	let {
		message,
		onsubmit,
		oncancal
	}: { message: string; onsubmit: (password: string) => void; oncancal?: () => void } = $props();

	let password = $state('');
	let passwordCheck = $state('');

	let bounceKey = $state(0);

	let matched = $derived(password.length != 0 && password == passwordCheck);
</script>

<form
	class="mb-4 items-center justify-between space-x-2 border-b border-outline pb-2 text-lg"
	onsubmit={() => {
		if (matched) onsubmit(password);
		else {
			bounceKey += 1;
			passwordCheck = '';
		}
	}}
>
	<div class="mb-2 flex justify-center">
		{message}
	</div>

	<div class="flex grow flex-row">
		<div>
			<input
				type="password"
				class="mb-1 w-full rounded-md border border-outline p-1"
				bind:value={password}
			/>
			<input
				type="password"
				class="w-full rounded-md border border-outline p-1"
				bind:value={passwordCheck}
			/>
		</div>
		<button class="mx-1 rounded-md p-1 hover:bg-hover{matched ? '' : ' hidden'}" type="submit">
			<CheckLine />
		</button>
		{#if !matched}
			<button class="hover:bg-hover mx-1 rounded-md p-1" onclick={oncancal}>
				<X />
			</button>
		{/if}
	</div>
</form>

{#if bounceKey != 0}
	{#key bounceKey}
		<div in:fade={{ duration: 300 }}>
			<Warning>TODO: fill the translation</Warning>
		</div>
	{/key}
{/if}
