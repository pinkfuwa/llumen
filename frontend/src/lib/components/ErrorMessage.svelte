<script lang="ts">
	import { dismissError, useError } from '$lib/error';
	import { CircleX } from '@lucide/svelte';
	import { fade } from 'svelte/transition';

	let error = useError();
</script>

{#if $error != null}
	<button
		class="fixed top-0 right-0 z-20 m-3 rounded-md border border-outline bg-component-bg px-3 py-2 text-left"
		in:fade={{ duration: 150 }}
		out:fade={{ duration: 150 }}
		onmouseleave={dismissError}
		onclick={dismissError}
	>
		<div class="mb-2 flex items-center">
			<CircleX class="mr-2 inline-block" />
			{$error.error} error
		</div>
		{#if $error.reason}
			<div class="max-w-sm lg:max-w-lg">
				{$error.reason}
			</div>
		{/if}
	</button>
{/if}
