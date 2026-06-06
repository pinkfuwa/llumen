<script lang="ts">
	let { text, block = false }: { text: string; block?: boolean } = $props();
</script>

{#if block}
	<div
		class="pointer-events-none h-px w-px overflow-hidden p-0 opacity-0 select-text"
		aria-hidden="true"
		style="margin: -1px;"
	>
		{text}
	</div>
{:else}
	<span
		class="pointer-events-none inline-block h-px w-px overflow-hidden p-0 opacity-0 select-text"
		aria-hidden="true"
		style="margin: -1px;">{text}</span
	>
{/if}
