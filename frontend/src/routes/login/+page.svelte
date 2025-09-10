<script lang="ts">
	import { goto } from '$app/navigation';
	import { Login } from '$lib/api/auth';
	import TiltBtn from '$lib/components/buttons/TiltBtn.svelte';
	import { page } from '$app/state';
	import { _ } from 'svelte-i18n';

	let username = $state('');
	let password = $state('');

	let { mutate, isPending, isError } = Login();

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
			(_) => {
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
</script>

<svelte:head>
	<title>{$_('login.title')}</title>
</svelte:head>
<main class="flex h-screen flex-col items-center justify-center">
	<h2 class="mb-3 bg-gradient-to-r from-dark to-blue-600 bg-clip-text text-4xl text-transparent">
		{$_('login.welcome')}
	</h2>
	<p class="text-md mb-3 font-serif">
		{$_('login.description')}
	</p>
	<div class="min-w-lg items-center rounded-lg p-6">
		<form
			class="flex flex-col items-center justify-center text-xl"
			onsubmit={handleSubmit}
			inert={$isPending}
		>
			<div class="mb-2 flex w-full justify-between">
				<label for="username" class="mr-3 min-w-[120px] text-center">
					{$_('login.username')}
				</label>
				<input
					type="text"
					placeholder="admin"
					id="username"
					class="login mb-2 grow border-b border-outline pb-2"
					bind:value={username}
					required
				/>
			</div>
			<div class="mb-6 flex w-full justify-between">
				<label for="password" class="mr-3 min-w-[120px] text-center">
					{$_('login.password')}
				</label>
				<input
					type="password"
					placeholder="P@88w0rd"
					id="password"
					class="login mb-2 grow border-b border-outline pb-2"
					bind:value={password!}
					required
				/>
			</div>

			<TiltBtn
				type="submit"
				disabled={$isPending}
				class="rounded-full border border-outline px-12 py-2 hover:bg-hover disabled:bg-hover"
			>
				{#if $isError}
					{$_('login.retry')}
				{:else if $isPending}
					{$_('login.loading')}
				{:else}
					{$_('login.submit')}
				{/if}
			</TiltBtn>
		</form>
	</div>
</main>

<style>
	input.login:focus-visible {
		outline: none;
	}
</style>
