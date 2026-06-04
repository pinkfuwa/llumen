<script lang="ts">
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { preference } from '$lib/preference/index.svelte';
	import Option from '../Option.svelte';
	import PasswordSection from '../PasswordSection.svelte';
	import Toggle from '../Toggle.svelte';
</script>

<div class="flex h-full flex-col overflow-auto">
	<div class="flex flex-col gap-2">
		<Option
			title={m['setting.theme'](lang)}
			data={[
				{ value: 'llumen', label: 'Llumen' },
				{ value: 'dracula', label: 'Dracula' },
				{ value: 'vitesse', label: 'Vitesse' }
			]}
			bind:selected={preference.value.theme.name}
		/>

		<Toggle
			title={m['setting.color_scheme'](lang)}
			trueLabel={m['setting.dark'](lang)}
			falseLabel={m['setting.light'](lang)}
			bind:value={preference.value.theme.dark}
		/>

		<Toggle
			title={m['setting.pattern'](lang)}
			trueLabel={m['setting.enable'](lang)}
			falseLabel={m['setting.disable'](lang)}
			bind:value={preference.value.theme.pattern}
		/>

		<Option
			title={m['setting.language'](lang)}
			data={[
				{ value: 'en', label: 'English' },
				{ value: 'zh-tw', label: '繁體中文' },
				{ value: 'zh-cn', label: '簡體中文' }
			]}
			bind:selected={preference.value.locale}
		/>

		<Option
			title={m['setting.enter'](lang)}
			data={[
				{ value: 'true', label: m['setting.enable'](lang) },
				{ value: 'false', label: m['setting.disable'](lang) }
			]}
			bind:selected={preference.value.submit_on_enter}
		/>

		<PasswordSection />
	</div>
</div>
