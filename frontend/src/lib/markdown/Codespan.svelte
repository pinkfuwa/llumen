<script>
	import { copy } from '$lib/copy';

	let { raw } = $props();

	let text = raw.replace(/`/g, '');

	let copied = $state(false);
</script>

<span class="py-1">
	<button
		class="my-[1px] rounded-md bg-background px-2 py-[2px] font-mono hover:bg-hover"
		onclick={() => copy(text)}>{text}</button
	>
</span>
