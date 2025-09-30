<script lang="ts">
	import { toHtml } from './latex';

	let { text, displayMode = false } = $props<{ text: string; displayMode?: boolean }>();
</script>

{#await toHtml(text, displayMode) then html}
	{@html html}
{/await}
