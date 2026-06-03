<script lang="ts">
	import { chatrooms, chatroomElement } from '$lib/api';
	import Entry from './Entry.svelte';
</script>

<ul
	class="nobar min-w-0 grow snap-y space-y-1 overflow-y-auto text-sm"
	bind:this={chatroomElement.val}
>
	{#each chatrooms.val as chatroom (chatroom.id)}
		<Entry bind:name={chatroom.name} id={chatroom.id} />
	{/each}
</ul>
