<script lang="ts">
	import { goto } from '$app/navigation';
	import { token } from '$lib/store';
	import { Login } from '$lib/api/user';

	let username = $state('');
	let password = $state('');

	let disable = $state(false);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		disable = true;

		const newToken: string | undefined = await Login(username, password);

		token.set(newToken == undefined ? '' : newToken);

		disable = false;
	}

	token.subscribe((value) => {
		if (value) {
			goto('/chat/new');
		}
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
					bind:value={username}
					required
				/>
			</div>
			<div class="mb-6 flex w-full justify-between">
				<label for="password" class="mr-3 min-w-[120px] text-center">Password</label>
				<input
					type="password"
					placeholder="P@88w0rd"
					class="login mb-2 grow border-b border-outline pb-2"
					bind:value={password}
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
