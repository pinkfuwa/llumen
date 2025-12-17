<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { page } from '$app/state';

	const messages: Record<string, string> = {
		'404': $_('error.page_not_found'),
		'500': $_('error.internal_server_error'),
		'403': $_('error.access_denied'),
		'401': $_('error.unauthorized'),
		'400': $_('error.bad_request'),
		'405': $_('error.method_not_allowed')
	};
</script>

<div class="flex h-screen flex-col items-center justify-center bg-login-bg">
	<h1 class="mb-4 text-4xl">
		{page.status}: {messages[String(page.status)] || $_('error.unknown_error')}
	</h1>
	<a href="/" class="text-2xl">{$_('error.back_to_home')}</a>
</div>
