<script lang="ts">
	import { TiltBtn } from '$lib/components';
	import Warning from '$lib/components/setting/Warning.svelte';
	import { _ } from 'svelte-i18n';
</script>

<Warning message={$_('setting.config_override_warning')} />
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
