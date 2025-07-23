<script lang="ts">
	import { Trash, CheckLine } from 'lucide-svelte';
	import { m } from '$lib/paraglide/messages';

	let nameBuffer = $state('');
	let username: undefined | string = $state(undefined);
	let passwordBuffer = $state('');
</script>

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

	<ul class="grid grid-cols-1 gap-2 text-lg lg:grid-cols-2 2xl:grid-cols-3">
		<li class="flex items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4">
			User 1 <Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
		</li>
		<li class="flex items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4">
			User 2 <Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
		</li>
		<li class="flex items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4">
			User 3 <Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
		</li>
		<li class="flex items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4">
			User 4 <Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
		</li>
		<li class="flex items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4">
			User 5 <Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
		</li>
	</ul>
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
					}
				}}><CheckLine /></button
			>
		</div>
	</div>
{/if}
