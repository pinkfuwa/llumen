<script lang="ts">
	import { _ } from 'svelte-i18n';

	import { CheckLine, X } from '@lucide/svelte';
	import { theme, locale, submitOnEnter } from '$lib/preference';
	import CheckPwd from '$lib/components/setting/CheckPwd.svelte';
	import { UpdateUser } from '$lib/api/user';
	import { get } from 'svelte/store';
	import Warning from '$lib/components/setting/Warning.svelte';
	import type { UserPreference } from '$lib/api/types';
	import { clearCache } from '$lib/api/state';
	import { token } from '$lib/store';
	import { goto } from '$app/navigation';

	let func = $state<'checkPwd' | 'setting'>('setting');
	let password = $state('');

	let message = $state('');

	let themeData = $state(get(theme));
	let localeData = $state(get(locale));
	let submitOnEnterData = $state(get(submitOnEnter));

	let { mutate, isPending, isError } = UpdateUser();

	function mutatePreference(preference: UserPreference) {
		message = 'error syncing preference';
		mutate({ preference });
	}
</script>

{#if func == 'setting'}
	{#if $isError}
		<Warning>
			{message}
		</Warning>
	{/if}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="theme" class="grow">{$_('setting.theme')}: </label>
		<select
			id="theme"
			bind:value={themeData}
			class="mx-1 rounded-md p-1 text-right hover:bg-hover"
			onchange={() => mutatePreference({ theme: themeData })}
			disabled={$isPending}
		>
			<option value="light">Modern Light</option>
			<option value="dark">Modern Dark</option>
			<option value="orange">Orange Light</option>
			<option value="blue">Blue Dark</option>
			<!-- <option value="custom">Custom Color</option> -->
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="lang" class="grow">{$_('setting.language')}: </label>
		<select
			id="lang"
			bind:value={localeData}
			class="mx-1 rounded-md p-1 text-right hover:bg-hover"
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
			class="mx-1 rounded-md p-1 text-right hover:bg-hover"
			onchange={() => mutatePreference({ submit_on_enter: submitOnEnterData })}
			disabled={$isPending}
		>
			<option value="true">{$_('setting.enable')}</option>
			<option value="false">{$_('setting.disable')}</option>
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="password" class="grow">{$_('setting.change_password')}: </label>
		<div class="flex items-center justify-between">
			<input
				type="password"
				id="password"
				class="rounded-md border border-outline p-1 text-right"
				bind:value={password}
				placeholder={$_('setting.old_password')}
			/>
			<button
				class="mx-1 rounded-md p-1 hover:bg-hover"
				onclick={() => {
					if (password.length > 0) func = 'checkPwd';
				}}><CheckLine /></button
			>
		</div>
	</div>
{:else}
	<CheckPwd
		message="Enter new password"
		onsubmit={(password) => {
			message = 'error updating password';
			mutate({ password }, () => {
				token.set(undefined);
				clearCache();
				goto('/login');
			});
		}}
		oncancal={() => {
			func = 'setting';
			password = '';
		}}
	></CheckPwd>
{/if}
