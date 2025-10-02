<script>
	import { copy } from '$lib/copy';

	let { raw } = $props();

	let text = raw.replace(/`/g, '');
</script>

<span class="py-1">
	<button
		class="my-0.5 cursor-pointer rounded-md bg-secondary px-2 py-0.5 font-mono break-all text-text duration-150 hover:bg-primary hover:text-text-hover"
		onclick={() => copy(text)}>{text}</button
	>
</span>
