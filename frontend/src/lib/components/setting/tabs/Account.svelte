<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { preference } from '$lib/preference/index.svelte';
	import Option from '../Option.svelte';
	import PasswordSection from '../PasswordSection.svelte';
	import Toggle from '../Toggle.svelte';
</script>

<div class="flex h-full flex-col overflow-auto">
	<div class="flex flex-col gap-2">
		<Option
			title={m['setting.theme']()}
			data={[
				{ value: 'llumen', label: 'Llumen' },
				{ value: 'dracula', label: 'Dracula' },
				{ value: 'vitesse', label: 'Vitesse' }
			]}
			bind:selected={preference.value.theme.name}
		/>

		<Toggle
			title={m['setting.color_scheme']()}
			trueLabel={m['setting.dark']()}
			falseLabel={m['setting.light']()}
			bind:value={preference.value.theme.dark}
		/>

		<Toggle
			title={m['setting.pattern']()}
			trueLabel={m['setting.enable']()}
			falseLabel={m['setting.disable']()}
			bind:value={preference.value.theme.pattern}
		/>

		<Option
			title={m['setting.language']()}
			data={[
				{ value: 'en', label: 'English' },
				{ value: 'zh-tw', label: '繁體中文' },
				{ value: 'zh-cn', label: '簡體中文' }
			]}
			bind:selected={preference.value.locale}
		/>

		<Option
			title={m['setting.enter']()}
			data={[
				{ value: 'true', label: m['setting.enable']() },
				{ value: 'false', label: m['setting.disable']() }
			]}
			bind:selected={preference.value.submit_on_enter}
		/>

		<PasswordSection />
	</div>
</div>
