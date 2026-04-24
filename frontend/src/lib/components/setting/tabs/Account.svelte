<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { theme, locale, submitOnEnter } from '$lib/preference';
	import { updateUser } from '$lib/api/user.svelte';
	import { get } from 'svelte/store';
	import Warning from '$lib/components/setting/Warning.svelte';
	import type { UserPreference } from '$lib/api/types';
	import Option from '../Option.svelte';
	import PasswordSection from '../PasswordSection.svelte';

	let themeData = $state(get(theme));
	let localeData = $state(get(locale));
	let submitOnEnterData = $state(get(submitOnEnter));

	let { mutate, isPending, isError } = updateUser();

	function mutatePreference(preference: UserPreference) {
		mutate({ preference });
	}

	function handleChange(key: string, value: string) {
		mutatePreference({ [key]: value });
	}
</script>

{#if isError()}
	<Warning>{$_('setting.account.error_sync_preference')}</Warning>
{/if}

<div class="flex h-full flex-col overflow-auto">
	<div class="flex flex-col gap-2">
		<Option
			title={$_('setting.theme')}
			data={[
				{ value: 'light', label: 'Llumen' },
				{ value: 'light-pattern', label: 'Llumen*' },
				{ value: 'dark', label: 'Sun set' },
				{ value: 'dark-pattern', label: 'Sun set*' },
				{ value: 'blue', label: 'Ocean' }
			]}
			bind:selected={themeData}
			disabled={isPending()}
			onchange={() => handleChange('theme', themeData)}
		/>

		<Option
			title={$_('setting.language')}
			data={[
				{ value: 'en', label: 'English' },
				{ value: 'zh-tw', label: '繁體中文' },
				{ value: 'zh-cn', label: '簡體中文' }
			]}
			bind:selected={localeData}
			disabled={isPending()}
			onchange={() => handleChange('locale', localeData)}
		/>

		<Option
			title={$_('setting.enter')}
			data={[
				{ value: 'true', label: $_('setting.enable') },
				{ value: 'false', label: $_('setting.disable') }
			]}
			bind:selected={submitOnEnterData}
			disabled={isPending()}
			onchange={() => handleChange('submit_on_enter', submitOnEnterData)}
		/>

		<PasswordSection />
	</div>
</div>
