<script lang="ts">
	import { error } from '$lib/error.svelte';
	import { CircleX } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import '$lib/preference/index.svelte';
	import '$lib/api/auth.svelte';

	const errorStyle =
		'fixed top-0 right-0 z-50 m-3 rounded-md border border-border bg-card px-3 py-2 text-left';
	const dismissError = () => {
		error.val = null;
	};
</script>

{#if error.val != null}
	<button
		class={errorStyle}
		in:fade={{ duration: 150 }}
		out:fade={{ duration: 150 }}
		onmouseleave={dismissError}
		onclick={dismissError}
	>
		<div class="mb-2 flex items-center">
			<CircleX class="mr-2 inline-block" />
			{error.val.error} error
		</div>
		{#if error.val.reason}
			<div class="max-w-sm lg:max-w-lg">
				{error.val.reason}
			</div>
		{/if}
	</button>
{/if}
