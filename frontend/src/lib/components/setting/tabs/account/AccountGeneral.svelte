<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { theme, locale, submitOnEnter } from '$lib/preference';
	import { updateUser } from '$lib/api/user.svelte';
	import { get } from 'svelte/store';
	import Warning from '$lib/components/setting/Warning.svelte';
	import type { UserPreference } from '$lib/api/types';
	import Select from '$lib/ui/Select.svelte';

	let themeData = $state(get(theme));
	let localeData = $state(get(locale));
	let submitOnEnterData = $state(get(submitOnEnter));

	let { mutate, isPending, isError } = updateUser();

	function mutatePreference(preference: UserPreference) {
		mutate({ preference });
	}
	$inspect('themeData', themeData);
</script>

{#if isError()}
	<Warning>{$_('setting.account.error_sync_preference')}</Warning>
{/if}
<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<label for="theme" class="grow">{$_('setting.theme')}: </label>
	<Select
		data={[
			{ value: 'light', label: 'Llumen' },
			{ value: 'light-pattern', label: 'Llumen*' },
			{ value: 'dark', label: 'Sun set' },
			{ value: 'dark-pattern', label: 'Sun set*' },
			{ value: 'blue', label: 'Ocean' }
		]}
		fallback="Select Theme"
		bind:selected={themeData}
		disabled={isPending()}
		class="w-36 truncate"
		popupClass="w-38"
		onchange={() => mutatePreference({ theme: themeData })}
	/>
</div>

<!-- class="grow truncate"
		popupClass="w-30" -->
<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<label for="lang" class="grow">{$_('setting.language')}: </label>
	<Select
		data={[
			{ value: 'en', label: 'English' },
			{ value: 'zh-tw', label: '繁體中文' }
		]}
		bind:selected={localeData}
		disabled={isPending()}
		class="w-36 truncate"
		popupClass="w-38"
		onchange={() => mutatePreference({ locale: localeData })}
	/>
</div>

<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<label for="enter" class="grow">{$_('setting.enter')}: </label>
	<Select
		data={[
			{ value: 'true', label: $_('setting.enable') },
			{ value: 'false', label: $_('setting.disable') }
		]}
		bind:selected={submitOnEnterData}
		disabled={isPending()}
		class="w-36 truncate"
		popupClass="w-38"
		onchange={() => mutatePreference({ submit_on_enter: submitOnEnterData })}
	/>
</div>
