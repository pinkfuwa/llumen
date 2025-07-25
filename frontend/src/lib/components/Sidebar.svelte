<script lang="ts">
	import { ArrowLeft, Settings, Plus, ArrowRight, Bot, Trash2 } from '@lucide/svelte';
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

			<ul class="nobar max-h-[calc(100vh-185px)] overflow-y-scroll text-sm">
				<li>
					<a
						href="/chat/new"
						class="mb-2 flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold hover:bg-hover"
					>
						<Plus class="mr-2 h-5 w-5" />
						New
					</a>
				</li>
				<li class="group flex items-center justify-between rounded-sm p-1.5 hover:bg-hover">
					<form
						class="grow"
						onsubmit={(e) => {
							e.preventDefault();
							console.log('Submitted');
						}}
					>
						<input
							class="editor h-6 w-full truncate"
							value="Recent breakthrough in LLM and Machine Learning"
							contenteditable="plaintext-only"
						/>
					</form>

					<Trash2 class="hidden p-[2px] group-hover:block" />
				</li>
				<li class="group flex items-center justify-between rounded-sm p-1.5 hover:bg-hover">
					<div class="h-6 w-full grow truncate select-none">
						Recent breakthrough in LLM and Machine Learning
					</div>
					<Trash2 class="hidden p-[2px] group-hover:block" />
				</li>
			</ul>
		</div>

		<div class="mt-4 border-t border-outline pt-4">
			<a
				class="flex w-full items-center justify-between rounded-lg border border-outline bg-white px-3 py-2 hover:bg-hover"
				href="/setting/openrouter"
			>
				<span class="font-medium text-gray-800">Admin</span>
				<Settings class="hover:text-dark" />
			</a>
		</div>
	</div>
{/if}
