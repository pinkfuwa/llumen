<script lang="ts">
	import { token } from '$lib/store';
	import { Login } from '$lib/api/user';
	import { goto } from '$app/navigation';

	let usernameBuf = $state('');
	let passwordBuf = $state('');

	let username = $state('');
	let password = $state('');

	let rawToken = Login(
		() => username,
		() => password
	);

	let disable = $derived($rawToken.isPending);

	const newToken = $derived($rawToken.data || '');
	$effect(() => {
		token().current = newToken;
	});

	function handleSubmit(event: Event) {
		event.preventDefault();
		username = usernameBuf;
		password = passwordBuf;
	}

	$effect(() => {
		if (token().current != '') goto('/chat/new');
	});
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<div class="mb-3 text-4xl">Welcome to llumen</div>
	<div class="text-md mb-3 font-extralight">
		Simple LLM chat frontend with great out-of-box experience.
	</div>
	<div class="min-w-lg items-center rounded-lg p-6">
		<form
			class="flex flex-col items-center justify-center text-xl"
			onsubmit={handleSubmit}
			inert={disable}
		>
			<div class="mb-2 flex w-full justify-between">
				<label for="username" class="mr-3 min-w-[120px] text-center">Username</label>
				<input
					type="text"
					placeholder="admin"
					class="login mb-2 grow border-b border-outline pb-2"
					bind:value={usernameBuf}
					required
				/>
			</div>
			<div class="mb-6 flex w-full justify-between">
				<label for="password" class="mr-3 min-w-[120px] text-center">Password</label>
				<input
					type="password"
					placeholder="P@88w0rd"
					class="login mb-2 grow border-b border-outline pb-2"
					bind:value={passwordBuf}
					required
				/>
			</div>

			<button
				type="submit"
				class="rounded-full border border-outline px-12 py-2 hover:bg-hover disabled:bg-hover"
				disabled={disable}
			>
				{disable ? 'Loading...' : 'Sign in'}
			</button>
		</form>
	</div>
</div>

<style>
	input.login:focus-visible {
		outline: none;
	}
</style>
