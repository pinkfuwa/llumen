<script lang="ts">
	import { ArrowLeft, Settings, Plus, ArrowRight, Bot } from 'lucide-svelte';
	import { slide } from 'svelte/transition';
	import { toggle } from './Setting/SettingState.svelte';

	let collapsed = $state(false);
</script>

{#if collapsed}
	<div class="fixed top-5 left-5">
		<button
			class="rounded-lg border border-outline bg-white p-2 text-dark shadow hover:bg-hover"
			onclick={() => (collapsed = false)}
		>
			<ArrowRight />
		</button>
	</div>
{:else}
	<div
		in:slide={{ duration: 180, axis: 'x' }}
		out:slide={{ duration: 180, axis: 'x' }}
		class="flex h-screen w-80 flex-col justify-between border-r border-outline bg-background p-5"
	>
		<div>
			<div class="mb-4 flex items-center justify-between border-b border-outline pb-1">
				<div class="flex items-center text-lg font-semibold">
					<Bot class="mx-2 h-6 w-6" /> <span>llumen</span>
				</div>
				<button class="rounded-lg p-3 text-dark hover:bg-hover" onclick={() => (collapsed = true)}>
					<ArrowLeft />
				</button>
			</div>

			<ul class="nobar max-h-[calc(100vh-200px)] overflow-y-scroll text-sm">
				<li
					class="mb-2 flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold hover:bg-hover"
				>
					<Plus class="mr-2 h-5 w-5" />
					New
				</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">
					Recent breakthrough in LLM and Machine Learning
				</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">
					Recent breakthrough in LLM and Machine Learning
				</li>
			</ul>
		</div>

		<div class="mt-4 border-t border-outline pt-4">
			<button
				class="flex w-full items-center justify-between rounded-lg border border-outline bg-white px-3 py-2 hover:bg-hover"
			>
				<span class="font-medium text-gray-800">Admin</span>
				<Settings class="hover:text-dark" onclick={toggle} />
			</button>
		</div>
	</div>
{/if}
