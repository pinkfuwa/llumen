<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { fade } from 'svelte/transition';
	import { X } from '@lucide/svelte';
	import { CircleUser, EthernetPort, LogOut, ShieldUser } from '@lucide/svelte';
	import { useToken } from '$lib/store';
	import { goto } from '$app/navigation';

	let token = useToken();
	let { children } = $props();
</script>

<div
	class="fixed top-0 left-0 z-4 flex h-screen w-screen items-center justify-center bg-black opacity-65"
	in:fade={{ duration: 180 }}
	out:fade={{ duration: 180 }}
></div>
<div class="fixed top-0 left-0 z-5 flex h-screen w-screen items-center justify-center">
	<div
		class="h-[calc(70vh-2rem)] w-[calc(80%-2rem)] overflow-y-scroll rounded-lg bg-background p-4 shadow-lg backdrop-opacity-100"
		in:fade={{ duration: 180 }}
		out:fade={{ duration: 180 }}
	>
		<div class="mb-2 flex items-center justify-between border-b border-outline px-2 text-xl">
			{m.setting()}
			<a class="left-0 p-3" href="/chat/new">
				<X />
			</a>
		</div>
		<div class="flex py-3 font-light">
			<ul class="mr-3 flex min-w-[210px] flex-col space-y-1 border-outline text-lg">
				<li class="rounded-md px-5 py-1 hover:bg-hover">
					<a href="/setting/account" class="flex items-center">
						<CircleUser class="mr-2 inline-block h-5 w-5" />
						{m.account_settings()}
					</a>
				</li>
				<li class="rounded-md px-5 py-1 hover:bg-hover">
					<a href="/setting/admin" class="flex items-center">
						<ShieldUser class="mr-2 inline-block h-5 w-5" />
						{m.admin_settings()}
					</a>
				</li>
				<li class="rounded-md px-5 py-1 hover:bg-hover">
					<a href="/setting/openrouter" class="flex items-center">
						<EthernetPort class="mr-2 inline-block h-5 w-5" /> Openrouter</a
					>
				</li>
				<li class="rounded-md px-5 py-1 hover:bg-hover">
					<button
						class="flex items-center"
						onclick={() => {
							token.set('');
							goto('/');
						}}
					>
						<LogOut class="mr-2 inline-block h-5 w-5" /> {m.logout()}</button
					>
				</li>
			</ul>
			<div class="grow px-2 pt-1">
				{@render children()}
			</div>
		</div>
	</div>
</div>
