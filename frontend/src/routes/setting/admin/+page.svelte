<script lang="ts">
	import { GetUsers, CreateUser, type User } from '$lib/user';
	import { Trash, CheckLine } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';

	let nameBuffer = $state('');
	let username: undefined | string = $state(undefined);
	let passwordBuffer = $state('');

	const userlistPromise = GetUsers();

	let createUserPromise: undefined | Promise<User> = $state(undefined);
</script>

{#if createUserPromise != undefined}
	{#await createUserPromise then user}
		<div class="mb-2 rounded-lg bg-red-700 hover:bg-red-500">
			<div class="ml-2 bg-background p-3 font-semibold hover:bg-hover">
				user <span class="rounded-md bg-hover p-2">{user.username}</span> created
			</div>
		</div>
	{:catch someError}
		<div class="mb-4 flex items-center justify-center border-b border-outline p-6 text-lg">
			System error: {someError.message}.
		</div>
	{/await}
{/if}
{#if username == undefined || (username as string).length == 0}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="name">{m.create_user()}: </label>
		<div class="flex items-center justify-between">
			<input
				type="text"
				id="name"
				class="rounded-md border border-outline p-1"
				bind:value={nameBuffer}
				placeholder={m.username()}
			/>
			<button
				class="mx-1 rounded-md p-1 hover:bg-hover"
				onclick={() => {
					if (nameBuffer.length > 0) {
						username = nameBuffer;
					}
				}}><CheckLine /></button
			>
		</div>
	</div>

	{#await userlistPromise}
		<div class="mb-4 flex items-center justify-center border-b border-outline p-6 text-lg">
			Loading users...
		</div>
	{:then users}
		<ul
			class="grid grid-cols-1 gap-2 border-b border-outline pb-2 text-lg lg:grid-cols-2 2xl:grid-cols-3"
		>
			{#each users as user}
				<li
					class="flex items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4"
				>
					{user.username}
					<Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
				</li>
			{/each}
		</ul>
	{:catch someError}
		<div class="mb-4 flex items-center justify-center border-b border-outline p-6 text-lg">
			System error: {someError.message}.
		</div>
	{/await}
{:else}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<div>Type password for <span class="rounded-md bg-hover p-2">{username}</span></div>
		<div class="flex items-center justify-between">
			<input
				type="text"
				id="password"
				class="rounded-md border border-outline p-1"
				bind:value={passwordBuffer}
			/>
			<button
				class="mx-1 rounded-md p-1 hover:bg-hover"
				onclick={() => {
					if (passwordBuffer.length > 0) {
						createUserPromise = CreateUser(username!, passwordBuffer);
					}
				}}><CheckLine /></button
			>
		</div>
	</div>
{/if}
