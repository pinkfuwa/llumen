<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { token } from '$lib/store';

	const messages: Record<string, string> = {
		'404': 'Page not found',
		'500': 'Internal server error',
		'403': 'Access denied',
		'401': 'Unauthorized',
		'400': 'Bad request',
		'405': 'Method not allowed'
	};

	$effect(() => {
		if (token().current == '') {
			goto('/login');
		}
	});
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<h1 class="text-4xl font-light">
		{$page.status}: {messages[String($page.status)] || 'Unknown error'}
	</h1>
</div>
