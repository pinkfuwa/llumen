<script lang="ts">
	let { params } = $props();
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<h1 class="text-4xl font-light">You are in chatroom id: {params.id}</h1>
	<a href="/chat/{+params.id + 1}" class="mt-3 text-3xl font-extralight"> Next Chatroom </a>
</div>
