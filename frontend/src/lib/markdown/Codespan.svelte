<script>
	let { raw } = $props();

	let text = raw.replace(/`/g, '');
</script>

<span class="py-1">
	<button
		class="rounded-md bg-background px-2 py-[2px] hover:bg-hover"
		onclick={() => {
			navigator.clipboard.writeText(text);
		}}>{text}</button
	>
</span>
