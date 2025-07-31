<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { page } from '$app/state';

	const messages: Record<string, string> = {
		'404': 'Page not found',
		'500': 'Internal server error',
		'403': 'Access denied',
		'401': 'Unauthorized',
		'400': 'Bad request',
		'405': 'Method not allowed'
	};
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<h1 class="mb-4 text-4xl font-light">
		{page.status}: {messages[String(page.status)] || 'Unknown error'}
	</h1>
	<a href="/chat/new" class="text-2xl">Back to home</a>
</div>
