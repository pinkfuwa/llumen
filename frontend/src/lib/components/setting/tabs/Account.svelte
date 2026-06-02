<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { preference } from '$lib/preference/index.svelte';
	import Option from '../Option.svelte';
	import PasswordSection from '../PasswordSection.svelte';
	import Toggle from '../Toggle.svelte';
</script>

<div class="flex h-full flex-col overflow-auto">
	<div class="flex flex-col gap-2">
		<Option
			title={$_('setting.theme')}
			data={[
				{ value: 'llumen', label: 'Llumen' },
				{ value: 'dracula', label: 'Dracula' },
				{ value: 'vitesse', label: 'Vitesse' }
			]}
			bind:selected={preference.value.theme.name}
		/>

		<Toggle
			title={$_('setting.color_scheme')}
			trueLabel={$_('setting.dark')}
			falseLabel={$_('setting.light')}
			bind:value={preference.value.theme.dark}
		/>

		<Toggle
			title={$_('setting.pattern')}
			trueLabel={$_('setting.enable')}
			falseLabel={$_('setting.disable')}
			bind:value={preference.value.theme.pattern}
		/>

		<Option
			title={$_('setting.language')}
			data={[
				{ value: 'en', label: 'English' },
				{ value: 'zh-tw', label: '繁體中文' },
				{ value: 'zh-cn', label: '簡體中文' }
			]}
			bind:selected={preference.value.locale}
		/>

		<Option
			title={$_('setting.enter')}
			data={[
				{ value: 'true', label: $_('setting.enable') },
				{ value: 'false', label: $_('setting.disable') }
			]}
			bind:selected={preference.value.submit_on_enter}
		/>

		<PasswordSection />
	</div>
</div>
