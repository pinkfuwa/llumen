<script>
	import CopyHint from './CopyHint.svelte';

	let { raw } = $props();

	let text = raw.replace(/`/g, '');

	let copied = $state(false);

	function copy() {
		navigator.clipboard.writeText(text);
		copied = true;
		setTimeout(() => {
			copied = false;
		}, 800);
	}
</script>

<span class="py-1">
	<button
		class="my-[1px] rounded-md bg-background px-2 py-[2px] font-mono hover:bg-hover"
		onclick={copy}>{text}</button
	>
</span>

{#if copied}
	<CopyHint />
{/if}
