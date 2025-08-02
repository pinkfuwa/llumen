<script lang="ts">
	import { _ } from 'svelte-i18n';

	import { fade } from 'svelte/transition';
	import { CheckLine, X } from '@lucide/svelte';
	import { theme, locale } from '$lib/store';

	let passwordBuffer = $state('');
	let checkPassword: undefined | string = $state(undefined);
</script>

{#if checkPassword == undefined || (checkPassword as string).length == 0}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="theme">{$_('setting.theme')}: </label>
		<select id="theme" bind:value={$theme} class="mx-1 rounded-md p-1 hover:bg-hover">
			<option value="light">Modern Light</option>
			<option value="dark">Eye-caring Dark</option>
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
		<label for="password">{$_('setting.change_password')}: </label>
		<div class="flex items-center justify-between">
			<input
				type="password"
				id="password"
				class="rounded-md border border-outline p-1"
				bind:value={passwordBuffer}
			/>
			<button
				class="mx-1 rounded-md p-1 hover:bg-hover"
				onclick={() => {
					if (passwordBuffer.length > 0) {
						checkPassword = passwordBuffer;
						passwordBuffer = '';
					}
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
				bind:value={passwordBuffer}
			/>

			{#if checkPassword == passwordBuffer}
				<button class="mx-1 rounded-md p-1 hover:bg-hover"><CheckLine /></button>
			{:else}
				<button
					class="mx-1 rounded-md p-1 hover:bg-hover"
					onclick={() => {
						checkPassword = '';
						passwordBuffer = '';
					}}><X /></button
				>
			{/if}
		</div>
	</div>
{/if}
