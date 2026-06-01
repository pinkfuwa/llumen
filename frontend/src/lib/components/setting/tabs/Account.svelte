<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { preference, propToRune } from '$lib/preference';
	import Option from '../Option.svelte';
	import PasswordSection from '../PasswordSection.svelte';
	import Toggle from '../Toggle.svelte';

	let themeName = propToRune(preference, 'theme', 'name') as { val: string };
	let dark = propToRune(preference, 'theme', 'dark') as { val: boolean };
	let pattern = propToRune(preference, 'theme', 'pattern') as { val: boolean };

	let locale = $state(propToRune(preference, 'locale'));
	let submitOnEnter = propToRune(preference, 'submit_on_enter');
</script>

<div class="flex h-full flex-col overflow-auto">
	<div class="flex flex-col gap-2">
		<Option
			title={$_('setting.theme')}
			data={[
				{ value: 'llumen', label: 'Llumen' },
				{ value: 'dracula', label: 'Dracula' },
				{ value: 'flexoki', label: 'Flexoki' },
				{ value: 'vitesse', label: 'Vitesse' }
			]}
			bind:selected={themeName.val}
		/>

		<Toggle
			title={$_('setting.color_scheme')}
			trueLabel={$_('setting.dark')}
			falseLabel={$_('setting.light')}
			bind:value={dark.val}
		/>

		<Toggle
			title={$_('setting.pattern')}
			trueLabel={$_('setting.enable')}
			falseLabel={$_('setting.disable')}
			bind:value={pattern.val}
		/>

		<Option
			title={$_('setting.language')}
			data={[
				{ value: 'en', label: 'English' },
				{ value: 'zh-tw', label: '繁體中文' },
				{ value: 'zh-cn', label: '簡體中文' }
			]}
			bind:selected={locale.val}
		/>

		<Option
			title={$_('setting.enter')}
			data={[
				{ value: 'true', label: $_('setting.enable') },
				{ value: 'false', label: $_('setting.disable') }
			]}
			bind:selected={submitOnEnter.val}
		/>

		<PasswordSection />
	</div>
</div>
