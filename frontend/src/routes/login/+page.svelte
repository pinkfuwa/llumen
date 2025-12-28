<script lang="ts">
	import { goto } from '$app/navigation';
	import { Login, TryHeaderAuth } from '$lib/api/auth';
	import { page } from '$app/state';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';
	import Input from '$lib/ui/Input.svelte';

	let username = $state('');
	let password = $state('');

	let { mutate, isPending, isError } = Login();
	let disabled = $derived(isPending() || username == '' || password == '');

	function handleSubmit(event: Event) {
		event.preventDefault();

		let usernameVal = username;
		let passwordVal = password;

		password = '';

		mutate(
			{
				username: usernameVal,
				password: passwordVal
			},
			(_data) => {
				const callback = page.url.searchParams.get('callback');

				if (callback) {
					let url = new URL(decodeURIComponent(callback), document.baseURI);
					if (url.origin == window.location.origin) goto(url);

					return;
				}

				goto('/chat/new');
			}
		);
	}

	$effect(() => {
		TryHeaderAuth();
	});
</script>

<svelte:head>
	<title>{$_('login.title')}</title>
</svelte:head>
<main class="flex h-screen flex-col items-center justify-center bg-login-bg">
	<h2
		class="mb-3 bg-gradient-to-r from-secondary to-primary bg-clip-text px-6 text-center text-4xl text-transparent"
	>
		{$_('login.welcome')}
	</h2>
	<p class="text-md mb-3 px-6 text-center font-serif">
		{$_('login.description')}
	</p>
	<div class="min-w-[80vw] items-center rounded-lg p-6 md:min-w-lg">
		<form class="grid grid-rows-3 gap-4" onsubmit={handleSubmit} inert={isPending()}>
			<div>
				<Input id="username" type="text" placeholder="admin" bind:value={username} required>
					{$_('login.username')}
				</Input>
			</div>
			<div>
				<Input type="password" placeholder="P@88w0rd" id="password" bind:value={password!} required>
					{$_('login.password')}
				</Input>
			</div>

			<Button type="submit" class="mt-4 text-lg" {disabled}>
				{#if isError()}
					{$_('login.retry')}
				{:else if isPending()}
					{$_('login.loading')}
				{:else}
					{$_('login.submit')}
				{/if}
			</Button>
		</form>
	</div>
</main>
