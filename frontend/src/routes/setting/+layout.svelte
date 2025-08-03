<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { fade } from 'svelte/transition';
	import { X } from '@lucide/svelte';
	import { CircleUser, EthernetPort, LogOut, ShieldUser } from '@lucide/svelte';
	import { token } from '$lib/store';
	import { goto } from '$app/navigation';
	import { clear as clearCache } from 'sswr';

	let { children } = $props();
</script>

<svelte:head>
	<title>{$_('setting.title')}</title>
</svelte:head>
<div
	class="fixed top-0 left-0 z-4 flex h-screen w-screen items-center justify-center bg-black opacity-65"
	in:fade={{ duration: 180 }}
	out:fade={{ duration: 180 }}
></div>
<div class="fixed top-0 left-0 z-5 flex h-screen w-screen items-center justify-center">
	<div
		class="h-[calc(70vh-2rem)] max-w-[750px] grow overflow-y-scroll rounded-lg bg-background p-4 shadow-lg backdrop-opacity-100 lg:max-w-[870px]"
		in:fade={{ duration: 180 }}
		out:fade={{ duration: 180 }}
	>
		<div class="mb-2 flex items-center justify-between border-b border-outline px-2 text-xl">
			{$_('setting.setting')}
			<a class="left-0 p-3" href="/">
				<X />
			</a>
		</div>
		<div class="flex py-3 font-light">
			<ul class="mr-3 flex min-w-[210px] flex-col space-y-1 border-outline text-lg">
				<li class="rounded-md px-5 py-1 hover:bg-hover">
					<a href="/setting/account" class="flex items-center">
						<CircleUser class="mr-2 inline-block h-5 w-5" />
						{$_('setting.account_settings')}
					</a>
				</li>
				<li class="rounded-md px-5 py-1 hover:bg-hover">
					<a href="/setting/admin" class="flex items-center">
						<ShieldUser class="mr-2 inline-block h-5 w-5" />
						{$_('setting.admin_settings')}
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
							token.set(undefined);
							clearCache();
							goto('/login');
						}}
					>
						<LogOut class="mr-2 inline-block h-5 w-5" /> {$_('setting.logout')}</button
					>
				</li>
			</ul>
			<div class="grow px-2 pt-1">
				{@render children()}
			</div>
		</div>
	</div>
</div>
