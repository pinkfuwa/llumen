<script lang="ts">
	let { text } = $props();

	let isBr = $derived.by(() => {
		if (text.length > 8) return false;
		let clean = text.replaceAll(' ', '');

		let brs = ['<br>', '<br/>', '</br>'];

		return brs.includes(clean);
	});
</script>

{#if isBr}
	<br />
{:else}
	{text}
{/if}
