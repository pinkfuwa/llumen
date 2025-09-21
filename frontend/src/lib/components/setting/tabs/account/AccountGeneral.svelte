<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { theme, locale, submitOnEnter } from '$lib/preference';
	import { UpdateUser } from '$lib/api/user';
	import { get } from 'svelte/store';
	import Warning from '$lib/components/setting/Warning.svelte';
	import type { UserPreference } from '$lib/api/types';

	let themeData = $state(get(theme));
	let localeData = $state(get(locale));
	let submitOnEnterData = $state(get(submitOnEnter));

	let { mutate, isPending, isError } = UpdateUser();

	function mutatePreference(preference: UserPreference) {
		mutate({ preference });
	}
</script>

{#if $isError}
	<Warning>{ $_('setting.account.error_sync_preference') }</Warning>
{/if}
<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<label for="theme" class="grow">{$_('setting.theme')}: </label>
	<select
		id="theme"
		bind:value={themeData}
		class="mx-1 rounded-md p-1 text-right duration-150 hover:bg-primary hover:text-text-hover"
		onchange={() => mutatePreference({ theme: themeData })}
		disabled={$isPending}
	>
		<option value="light">Llumen</option>
	</select>
</div>

<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<label for="lang" class="grow">{$_('setting.language')}: </label>
	<select
		id="lang"
		bind:value={localeData}
		class="mx-1 rounded-md p-1 text-right duration-150 hover:bg-primary hover:text-text-hover"
		onchange={() => mutatePreference({ locale: localeData })}
		disabled={$isPending}
	>
		<option value="en">English</option>
		<option value="zh-tw">繁體中文</option>
	</select>
</div>

<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<label for="enter" class="grow">{$_('setting.enter')}: </label>
	<select
		id="enter"
		bind:value={submitOnEnterData}
		class="mx-1 rounded-md p-1 text-right duration-150 hover:bg-primary hover:text-text-hover"
		onchange={() => mutatePreference({ submit_on_enter: submitOnEnterData })}
		disabled={$isPending}
	>
		<option value="true">{$_('setting.enable')}</option>
		<option value="false">{$_('setting.disable')}</option>
	</select>
</div>