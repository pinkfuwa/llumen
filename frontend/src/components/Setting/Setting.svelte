<script lang="ts">
	import { fade } from 'svelte/transition';
	import { getCollapsed, toggle } from './SettingState.svelte';
	import { X } from 'lucide-svelte';
	import Appearance from './Appearance.svelte';
	import Openrouter from './Openrouter.svelte';
	import { Eclipse, EthernetPort, LogOut } from 'lucide-svelte';

	type tab = 'appearance' | 'openrouter' | 'logout';
	let currentTab = 'appearance';
</script>

{#if !getCollapsed()}
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
				Settings
				<button class="left-0 p-3" on:click={toggle}>
					<X />
				</button>
			</div>
			<div class="flex py-3 font-light">
				<ul class="mr-3 flex min-w-[190px] flex-col space-y-1 border-outline text-lg">
					<li class="rounded-md px-5 py-1 hover:bg-hover">
						<button on:click={() => (currentTab = 'appearance')} class="flex items-center">
							<Eclipse class="mr-2 inline-block h-5 w-5" /> Appearance</button
						>
					</li>
					<li class="rounded-md px-5 py-1 hover:bg-hover">
						<button on:click={() => (currentTab = 'openrouter')} class="flex items-center">
							<EthernetPort class="mr-2 inline-block h-5 w-5" /> Openrouter</button
						>
					</li>
					<li class="rounded-md px-5 py-1 hover:bg-hover">
						<button class="flex items-center">
							<LogOut class="mr-2 inline-block h-5 w-5" /> Logout</button
						>
					</li>
				</ul>
				<div class="grow px-2 pt-1">
					{#if currentTab === 'appearance'}
						<Appearance />
					{:else if currentTab === 'openrouter'}
						<Openrouter />
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}
