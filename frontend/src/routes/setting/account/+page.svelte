<script lang="ts">
	import { _ } from 'svelte-i18n';

	import { fade } from 'svelte/transition';
	import { CheckLine, X } from '@lucide/svelte';
	import { theme, locale, enterSubmit } from '$lib/store';

	let func = $state<'checkPwd' | 'setting'>('setting');
	let password = $state('');
	let checkPassword = $state('');
</script>

{#if func == 'setting'}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="theme">{$_('setting.theme')}: </label>
		<select id="theme" bind:value={$theme} class="mx-1 rounded-md p-1 text-center hover:bg-hover">
			<option value="light">Modern Light</option>
			<option value="dark">Modern Dark</option>
			<option value="orange">Orange Light</option>
			<option value="blue">&nbsp;&nbsp;Blue Dark</option>
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="lang">{$_('setting.language')}: </label>
		<select id="lang" bind:value={$locale} class="mx-1 rounded-md p-1 hover:bg-hover">
			<option value="en">English</option>
			<option value="zh-tw">繁體中文</option>
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="enter">{$_('setting.enter')}: </label>
		<select id="enter" bind:value={$enterSubmit} class="mx-1 rounded-md p-1 hover:bg-hover">
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
				class="rounded-md border border-outline p-1"
				bind:value={password}
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
	<div in:fade={{ duration: 180 }} class="flex flex-col space-y-2 text-lg">
		<label for="password">Type Password Again: </label>
		<div class="flex">
			<input
				type="password"
				id="password"
				class="grow rounded-md border border-outline p-1"
				bind:value={checkPassword}
			/>

			{#if checkPassword == password}
				<button class="mx-1 rounded-md p-1 hover:bg-hover"><CheckLine /></button>
			{:else}
				<button
					class="mx-1 rounded-md p-1 hover:bg-hover"
					onclick={() => {
						checkPassword = '';
						password = '';
						func = 'setting';
					}}><X /></button
				>
			{/if}
		</div>
	</div>
{/if}
