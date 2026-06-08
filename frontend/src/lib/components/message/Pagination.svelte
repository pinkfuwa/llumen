<script lang="ts">
	import { messages, syncMessage } from '$lib/api/message.svelte';
	import ResponseBox from './ResponseBox.svelte';
	import ResponseEdit from './ResponseEdit.svelte';
	import User from './User.svelte';
	import Chunks from './Chunks.svelte';

	async function handleUpdate(
		messageId: number,
		text: string,
		updatedFiles: Array<{ name: string; id: number }>
	) {
		// FIXME: there is only one messags array, which we can use data-as-API concept to avoid prop drilling
		await syncMessage(messageId, text, updatedFiles);
	}
</script>

{#each messages.val.toReversed() as msg (msg.id)}
	{@const streaming = msg.stream}
	{#if msg.inner.t == 'user'}
		{@const content = msg.inner.c.text}
		{@const files = msg.inner.c.files}
		<User
			{content}
			{files}
			messageId={msg.id}
			onupdate={(text, updatedFiles) => handleUpdate(msg.id, text, updatedFiles)}
		/>
	{:else if msg.inner.t == 'assistant'}
		{@const chunks = msg.inner.c}
		<ResponseBox>
			<Chunks {chunks} {streaming} />

			{#if streaming}
				<div class="space-y-4">
					<hr class="mx-3 animate-pulse rounded-md border-primary bg-primary p-1" />
					<hr class="mx-3 animate-pulse rounded-md border-primary bg-primary p-1" />
				</div>
			{:else}
				{@const text = chunks
					.filter((x) => x.t == 'text')
					.map((x) => x.c)
					.join('\n')
					.trim()}
				<ResponseEdit content={text} token={msg.token_count} cost={msg.price} />
			{/if}
		</ResponseBox>
	{/if}
{/each}
