<script lang="ts">
	import { m } from '$lib/paraglide/messages';
</script>

<div class="relative select-none">
	<footer
		class="absolute top-1 z-10 flex w-full items-center justify-center text-sm text-muted-foreground"
	>
		{m['chat.hallucination_warning']()}
	</footer>
</div>
