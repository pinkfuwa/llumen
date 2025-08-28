<script lang="ts">
	import { TiltBtn } from '$lib/components';
	import { _ } from 'svelte-i18n';
	const defaultConfig = 'openrouter_id="openai/gpt-oss-20b:free"\ndisplay_name="GPT-OSS 20B"';

	let config = $state(defaultConfig);
</script>

<!-- TODO: use shiki and create a overlay editor -->
<textarea
	class="min-h-[200px] w-full rounded-md border border-outline bg-light p-3"
	bind:value={config}
></textarea>
<TiltBtn
	class="mt-3 rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
	>{$_('setting.check_syntax')}</TiltBtn
>
<TiltBtn
	class="mt-3 ml-2 rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
	>{$_('setting.save_settings')}</TiltBtn
>
