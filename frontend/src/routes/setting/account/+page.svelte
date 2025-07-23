<script lang="ts">
	import { fade } from 'svelte/transition';
	import { CheckLine, X } from 'lucide-svelte';
	import { m } from '$lib/paraglide/messages';

	interface Preference {
		theme: 'light' | 'dark';
		language: 'en' | 'zh-tw';
	}

	import { storable } from '$lib/LocalStorage.svelte';
	let preference = storable({ theme: 'light', language: 'en' } as Preference);

	let passwordBuffer = $state('');
	let checkPassword: undefined | string = $state(undefined);
</script>

{#if checkPassword == undefined || (checkPassword as string).length == 0}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="theme">{m.theme()}: </label>
		<select id="theme" bind:value={$preference.theme} class="mx-1 rounded-md p-1 hover:bg-hover">
			<option value="light">Modern Light</option>
			<option value="dark">Eye-caring Dark</option>
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="lang">{m.language()}: </label>
		<select id="lang" bind:value={$preference.language} class="mx-1 rounded-md p-1 hover:bg-hover">
			<option value="en">English</option>
			<option value="zh-tw">繁體中文</option>
		</select>
	</div>

	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="password">{m.change_password()}: </label>
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
					}
				}}><CheckLine /></button
			>
		</div>
	</div>
{:else}
	<div in:fade={{ duration: 180 }} class="flex flex-col space-y-2 text-lg">
		<label for="password">Type Password Again: </label>
		<input type="password" id="password" class="w-full rounded-md border border-outline p-1" />

		<div class="">
			<button
				class="mx-1 rounded-md p-1 hover:bg-hover"
				onclick={() => {
					checkPassword = '';
					passwordBuffer = '';
				}}><X /></button
			>
			<button class="mx-1 rounded-md p-1 hover:bg-hover"><CheckLine /></button>
		</div>
	</div>
{/if}
