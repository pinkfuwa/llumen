<script lang="ts">
	import { _ } from 'svelte-i18n';
</script>

<div class="relative select-none">
	<footer class="absolute top-1 z-10 flex w-full items-center justify-center text-sm text-outline">
		{$_('chat.hallucination_warning')}
	</footer>
</div>
