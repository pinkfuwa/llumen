<script lang="ts">
	import { goto } from '$app/navigation';
	import { token } from '$lib/localStorage';
	const logined = token().current != undefined;
	goto(logined ? '/chat/new' : '/login');
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<h1 class="text-4xl font-light">Login is required</h1>
</div>
