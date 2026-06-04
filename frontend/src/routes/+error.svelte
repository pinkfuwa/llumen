<script lang="ts">
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { page } from '$app/state';

	const messages = $derived<Record<string, string>>({
		'404': m['error.page_not_found'](lang),
		'500': m['error.internal_server_error'](lang),
		'403': m['error.access_denied'](lang),
		'401': m['error.unauthorized'](lang),
		'400': m['error.bad_request'](lang),
		'405': m['error.method_not_allowed'](lang)
	});
</script>

<div class="bg-surface-base flex h-screen flex-col items-center justify-center">
	<h1 class="mb-4 text-4xl">
		{page.status}: {messages[String(page.status)] || m['error.unknown_error'](lang)}
	</h1>
	<a href="/" class="text-2xl">{m['error.back_to_home'](lang)}</a>
</div>
