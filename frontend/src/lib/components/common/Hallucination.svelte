<script lang="ts">
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
</script>

<div class="relative select-none">
	<footer
		class="absolute top-1 z-10 flex w-full items-center justify-center text-sm text-muted-foreground"
	>
		{m['chat.hallucination_warning'](lang)}
	</footer>
</div>
