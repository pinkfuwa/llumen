<script lang="ts">
	import { messages } from '$lib/api';
</script>

<div class="fixed top-0 right-0 z-10 hidden h-full lg:block">
	<div class="nobar h-full space-y-3 overflow-y-scroll py-6 pl-18">
		{#each messages.val.toReversed() as msg (msg.id)}
			{#if msg.inner.t == 'user'}
				<a
					class="block h-32 w-5 rounded-l-lg border border-border bg-card hover:bg-interactive-hover focus:ring-ring focus:outline-none"
					aria-label="button"
					href={'#' + msg.id}
				></a>
			{/if}
		{/each}
	</div>
</div>
