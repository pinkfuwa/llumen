<script lang="ts">
	let { top = false } = $props();
</script>

<div class="relative select-none">
	<footer
		class="absolute {top
			? 'top-0'
			: 'bottom-1'} z-10 flex w-full items-center justify-center text-sm text-outline"
	>
		<a
			href="https://github.com/pinkfuwa/llumen"
			target="_blank"
			class="rounded-md px-2 py-0.5 hover:bg-primary hover:text-text-hover duration-150">llumen</a
		>Made by<a
			href="https://github.com/pinkfuwa"
			target="_blank"
			class="rounded-md px-2 py-0.5 hover:bg-primary hover:text-text-hover duration-150">pinkfuwa</a
		>&COPY; {new Date().getFullYear()}
	</footer>
</div>
