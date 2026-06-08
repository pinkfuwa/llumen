<script lang="ts">
	import { Login } from '$lib/api';
	import Button from '$lib/ui/Button.svelte';
	import Input from '$lib/ui/Input.svelte';
	import type { MutationStatus } from '$lib/api';
	import { t } from 'svelte-intl-precompile';

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
	<title>{$t('login.title')}</title>
</svelte:head>
<main class="bg-surface-base flex h-screen flex-col items-center justify-center">
	<h2
		class="mb-3 bg-gradient-to-r from-primary to-primary/50 bg-clip-text px-6 text-center text-4xl text-transparent"
	>
		{$t('login.welcome')}
	</h2>
	<p class="text-md mb-3 px-6 text-center font-serif">
		{$t('login.description')}
	</p>
	<div class="min-w-[80vw] items-center rounded-lg p-6 md:min-w-lg">
		<form class="grid grid-rows-3 gap-4" onsubmit={handleSubmit} inert={pending}>
			<div>
				<Input id="username" type="text" placeholder="admin" bind:value={username} required>
					{$t('login.username')}
				</Input>
			</div>
			<div>
				<Input type="password" placeholder="P@88w0rd" id="password" bind:value={password!} required>
					{$t('login.password')}
				</Input>
			</div>

			<Button type="submit" class="mt-4 text-lg" {disabled}>
				{#if status == 'failed'}
					{$t('login.retry')}
				{:else if pending}
					{$t('login.loading')}
				{:else}
					{$t('login.submit')}
				{/if}
			</Button>
		</form>
	</div>
</main>
