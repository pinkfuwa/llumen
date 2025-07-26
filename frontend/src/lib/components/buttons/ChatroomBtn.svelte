<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	let { name = $bindable('Default chatroom title') } = $props();

	let editing = $state(false);

	function init(element: HTMLInputElement) {
		element.focus();
	}
</script>

<li class="group text-md flex items-center justify-between rounded-sm p-1.5 hover:bg-hover">
	{#if editing}
		<form
			class="grow items-center"
			onsubmit={(e) => {
				e.preventDefault();
				editing = false;
			}}
		>
			<input
				class="editor h-6 w-full truncate"
				bind:value={name}
				use:init
				contenteditable="plaintext-only"
			/>
		</form>

		<Trash2 class="hidden p-[2px] group-hover:block" />
	{:else}
		<button
			class="h-6 w-full grow items-center truncate text-left select-none"
			ondblclick={() => (editing = true)}
		>
			{name}
		</button>
		<Trash2 class="hidden p-[2px] group-hover:block" />
	{/if}
</li>
