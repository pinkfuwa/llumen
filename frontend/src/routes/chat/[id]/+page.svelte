<script lang="ts">
	let { params } = $props();
	import { _ } from 'svelte-i18n';
	import MessageInput from '$lib/components/MessageInput.svelte';
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<div class="flex h-screen grow flex-col items-center justify-center">
		<h1 class="text-4xl font-light">
			You are in chatroom id: {params.id}
		</h1>
	</div>
	<div class="mb-2">
		<MessageInput above />
	</div>
</div>
