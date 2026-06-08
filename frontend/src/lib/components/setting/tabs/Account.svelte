<script lang="ts">
	import { preference } from '$lib/preference/index.svelte';
	import Option from '../Option.svelte';
	import PasswordSection from '../PasswordSection.svelte';
	import Toggle from '../Toggle.svelte';
	import { t } from 'svelte-intl-precompile';
</script>

<div class="flex h-full flex-col overflow-auto">
	<div class="flex flex-col gap-2">
		<Option
			title={$t('setting.theme')}
			data={[
				{ value: 'llumen', label: 'Llumen' },
				{ value: 'dracula', label: 'Dracula' },
				{ value: 'vitesse', label: 'Vitesse' }
			]}
			bind:selected={preference.value.theme.name}
		/>

		<Toggle
			title={$t('setting.color_scheme')}
			trueLabel={$t('setting.dark')}
			falseLabel={$t('setting.light')}
			bind:value={preference.value.theme.dark}
		/>

		<Toggle
			title={$t('setting.pattern')}
			trueLabel={$t('setting.enable')}
			falseLabel={$t('setting.disable')}
			bind:value={preference.value.theme.pattern}
		/>

		<Option
			title={$t('setting.language')}
			data={[
				{ value: 'en', label: 'English' },
				{ value: 'zh-tw', label: '繁體中文' },
				{ value: 'zh-cn', label: '簡體中文' }
			]}
			bind:selected={preference.value.locale}
		/>

		<Option
			title={$t('setting.enter')}
			data={[
				{ value: 'true', label: $t('setting.enable') },
				{ value: 'false', label: $t('setting.disable') }
			]}
			bind:selected={preference.value.submit_on_enter}
		/>

		<PasswordSection />
	</div>
</div>
