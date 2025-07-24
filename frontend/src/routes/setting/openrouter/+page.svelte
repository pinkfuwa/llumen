<script lang="ts">
	import { m } from '$lib/paraglide/messages';
</script>

<div class="mb-2 rounded-lg bg-red-700 hover:bg-red-500">
	<div class="ml-2 bg-background p-3 font-semibold hover:bg-hover">
		{m.config_override_warning()}
	</div>
</div>
<div contenteditable="plaintext-only" class="min-h-[200px] rounded-md bg-hover p-3">
	text config
</div>
<button
	class="mt-3 rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
	>{m.check_syntax()}</button
>
<button
	class="mt-3 ml-2 rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
	>{m.save_settings()}</button
>
