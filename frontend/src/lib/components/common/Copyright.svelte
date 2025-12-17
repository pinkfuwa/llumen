<script lang="ts">
	import { _ } from 'svelte-i18n';
</script>

<div class="relative select-none">
	<footer
		class="absolute bottom-1 z-10 flex w-full items-center justify-center text-sm text-outline"
	>
		<a
			href="https://github.com/pinkfuwa/llumen"
			target="_blank"
			tabindex="-1"
			class="rounded-md px-2 py-0.5 duration-150 hover:bg-primary hover:text-text-hover">llumen</a
		>Made by<a
			href="https://github.com/pinkfuwa"
			target="_blank"
			tabindex="-1"
			class="rounded-md px-2 py-0.5 duration-150 hover:bg-primary hover:text-text-hover">pinkfuwa</a
		>&COPY; {new Date().getFullYear()}
	</footer>
</div>
