<script lang="ts">
	import { page } from '$app/state';
	import { t } from 'svelte-intl-precompile';

	const messages = $derived<Record<string, string>>({
		'404': $t('error.page_not_found'),
		'500': $t('error.internal_server_error'),
		'403': $t('error.access_denied'),
		'401': $t('error.unauthorized'),
		'400': $t('error.bad_request'),
		'405': $t('error.method_not_allowed')
	});
</script>

<div class="bg-surface-base flex h-screen flex-col items-center justify-center">
	<h1 class="mb-4 text-4xl">
		{page.status}: {messages[String(page.status)] || $t('error.unknown_error')}
	</h1>
	<a href="/" class="text-2xl">{$t('error.back_to_home')}</a>
</div>
