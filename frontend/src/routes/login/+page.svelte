<script lang="ts">
	import { goto } from '$app/navigation';
	import { Login } from '$lib/api/user';
	import { useToken } from '$lib/store';
	let username = $state('');
	let password = $state('');

	let loginMutation = Login();

	let token = useToken();

	function handleSubmit(event: Event) {
		event.preventDefault();
		$loginMutation.mutate(
			{
				username: username,
				password: password
			},
			{
				onSuccess: (data) => {
					console.log('Login successful', data);
					goto('/chat/new');
					token.set(data.token);
				}
			}
		);
	}
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
			inert={$loginMutation.isPending}
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
					bind:value={password!}
					required
				/>
			</div>

			<button
				type="submit"
				class="rounded-full border border-outline px-12 py-2 hover:bg-hover disabled:bg-hover"
				disabled={$loginMutation.isPending}
			>
				{#if $loginMutation.failureCount > 0}
					Try again
				{:else if $loginMutation.isPending}
					Loading...
				{:else}
					Sign in
				{/if}
			</button>
		</form>
	</div>
</div>

<style>
	input.login:focus-visible {
		outline: none;
	}
</style>
