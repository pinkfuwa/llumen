<script lang="ts">
	import { t } from 'svelte-intl-precompile';
</script>

<div class="relative select-none">
	<footer
		class="absolute top-1 z-10 flex w-full items-center justify-center text-sm text-muted-foreground"
	>
		{$t('chat.hallucination_warning')}
	</footer>
</div>
