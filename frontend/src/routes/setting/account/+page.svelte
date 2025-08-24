<script lang="ts">
	import { _ } from 'svelte-i18n';

	import { CheckLine, X } from '@lucide/svelte';
	import { theme, locale, enterSubmit } from '$lib/store';
	import CheckPwd from '$lib/components/setting/CheckPwd.svelte';
	import { UpdateUser } from '$lib/api/user';
	import { get } from 'svelte/store';
	import Warning from '$lib/components/setting/Warning.svelte';

	let func = $state<'checkPwd' | 'setting'>('setting');
	let password = $state('');

	let { mutate, isPending, isError } = UpdateUser();
</script>

{#if func == 'setting'}
	{#if $isError}
		<Warning message="error syncing perference" />
	{/if}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="theme">{$_('setting.theme')}: </label>
		<select
			id="theme"
			bind:value={$theme}
			class="mx-1 rounded-md p-1 text-center hover:bg-hover"
			onchange={() =>
				mutate({
					perference: {
						theme: get(theme)
					}
				})}
			disabled={$isPending}
		>
			<option value="light">Modern Light</option>
			<option value="dark">Modern Dark</option>
			<option value="orange">Orange Light</option>
			<option value="blue">&nbsp;&nbsp;Blue Dark</option>
			<!-- <option value="custom">Custom Color</option> -->
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="lang">{$_('setting.language')}: </label>
		<select
			id="lang"
			bind:value={$locale}
			class="mx-1 rounded-md p-1 hover:bg-hover"
			onchange={() =>
				mutate({
					perference: {
						locale: get(locale)
					}
				})}
			disabled={$isPending}
		>
			<option value="en">English</option>
			<option value="zh-tw">繁體中文</option>
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="enter">{$_('setting.enter')}: </label>
		<select
			id="enter"
			bind:value={$enterSubmit}
			class="mx-1 rounded-md p-1 hover:bg-hover"
			onchange={() =>
				mutate({
					perference: {
						submit_on_enter: get(enterSubmit)
					}
				})}
			disabled={$isPending}
		>
			<option value="true">{$_('setting.enable')}</option>
			<option value="false">{$_('setting.disable')}</option>
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="password">{$_('setting.change_password')}: </label>
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
			// TODO: change password
		}}
		oncancal={() => {
			func = 'setting';
			password = '';
		}}
	></CheckPwd>
{/if}
