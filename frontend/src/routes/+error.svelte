<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';

	const messages = $derived<Record<string, string>>({
		'404': m['error.page_not_found'](),
		'500': m['error.internal_server_error'](),
		'403': m['error.access_denied'](),
		'401': m['error.unauthorized'](),
		'400': m['error.bad_request'](),
		'405': m['error.method_not_allowed']()
	});
</script>

<div class="bg-surface-base flex h-screen flex-col items-center justify-center">
	<h1 class="mb-4 text-4xl">
		{page.status}: {messages[String(page.status)] || m['error.unknown_error']()}
	</h1>
	<a href="/" class="text-2xl">{m['error.back_to_home']()}</a>
</div>
