<script lang="ts">
	import { Login } from '$lib/api';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import Button from '$lib/ui/Button.svelte';
	import Input from '$lib/ui/Input.svelte';
	import type { MutationStatus } from '$lib/api';

	let username = $state('');
	let password = $state('');

	let status = $state<MutationStatus>('untried');

	let disabled = $derived(status == 'pending' || username == '' || password == '');

	let pending = $derived(status == 'pending' || status == 'success');

	async function handleSubmit(event: Event) {
		event.preventDefault();

		let usernameVal = username;
		let passwordVal = password;

		status = 'pending';

		status = await Login(usernameVal, passwordVal);

		password = '';
	}
</script>

<svelte:head>
	<title>{m['login.title'](lang)}</title>
</svelte:head>
<main class="bg-surface-base flex h-screen flex-col items-center justify-center">
	<h2
		class="mb-3 bg-gradient-to-r from-primary to-primary/50 bg-clip-text px-6 text-center text-4xl text-transparent"
	>
		{m['login.welcome'](lang)}
	</h2>
	<p class="text-md mb-3 px-6 text-center font-serif">
		{m['login.description'](lang)}
	</p>
	<div class="min-w-[80vw] items-center rounded-lg p-6 md:min-w-lg">
		<form class="grid grid-rows-3 gap-4" onsubmit={handleSubmit} inert={pending}>
			<div>
				<Input id="username" type="text" placeholder="admin" bind:value={username} required>
					{m['login.username'](lang)}
				</Input>
			</div>
			<div>
				<Input type="password" placeholder="P@88w0rd" id="password" bind:value={password!} required>
					{m['login.password'](lang)}
				</Input>
			</div>

			<Button type="submit" class="mt-4 text-lg" {disabled}>
				{#if status == 'failed'}
					{m['login.retry'](lang)}
				{:else if pending}
					{m['login.loading'](lang)}
				{:else}
					{m['login.submit'](lang)}
				{/if}
			</Button>
		</form>
	</div>
</main>
