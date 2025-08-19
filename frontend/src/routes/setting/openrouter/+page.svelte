<script lang="ts">
	import TiltBtn from '$lib/components/buttons/TiltBtn.svelte';
	import { _ } from 'svelte-i18n';
</script>

<div class="mb-2 rounded-lg bg-red-700 hover:bg-red-500">
	<div class="ml-2 bg-background p-3 font-semibold">
		{$_('setting.config_override_warning')}
	</div>
</div>
<div
	contenteditable="plaintext-only"
	class="min-h-[200px] rounded-md border border-outline bg-light p-3"
>
	text config
</div>
<TiltBtn
	class="mt-3 rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
	>{$_('setting.check_syntax')}</TiltBtn
>
<TiltBtn
	class="mt-3 ml-2 rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
	>{$_('setting.save_settings')}</TiltBtn
>
