<script lang="ts">
	import { ArrowLeft, Search, ChevronsUp, Plus, ArrowRight, Bot } from 'lucide-svelte';
	import { slide } from 'svelte/transition';

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

			<ul class="text-sm">
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">help me generate a business...</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">What's the best approach to...</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">What is the meaning of life?</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">What strategies can help manag...</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">
					Create a recipe for a fusion dish...
				</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">
					What specific metrics should I us...
				</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">Latest AI Updates</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">any oss</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">how to use vercel ai sdk</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">How does AI work?</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">latest news</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">Recent breakthrough in LLM and...</li>
				<li class="truncate rounded-sm p-1.5 hover:bg-hover">Recent AI Models and it's...</li>
				<li
					class="mt-4 flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold hover:bg-hover"
				>
					<Plus class="mr-2 h-5 w-5" />
					New
				</li>
			</ul>
		</div>

		<div class="mt-4 border-t border-outline pt-4">
			<button
				class="flex w-full items-center justify-between rounded-lg border border-outline bg-white px-3 py-2 hover:bg-hover"
			>
				<span class="font-medium text-gray-800">Admin</span>
				<ChevronsUp class="hover:text-dark" />
			</button>
		</div>
	</div>
{/if}
