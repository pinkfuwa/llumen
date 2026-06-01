<script lang="ts">
	import { _ } from 'svelte-i18n';

	const linkStyle = 'rounded-md px-2 py-0.5 duration-150 hover:bg-interactive-hover';
</script>

<div class="relative select-none">
	<footer
		class="absolute bottom-1 z-10 flex w-full items-center justify-center text-sm text-muted-foreground"
	>
		<a href="https://github.com/pinkfuwa/llumen" target="_blank" tabindex="-1" class={linkStyle}
			>llumen</a
		>Made by<a href="https://github.com/pinkfuwa" target="_blank" tabindex="-1" class={linkStyle}
			>pinkfuwa</a
		>&COPY; {new Date().getFullYear()}
	</footer>
</div>
