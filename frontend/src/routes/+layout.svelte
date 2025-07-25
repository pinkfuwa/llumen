<script lang="ts">
	import '../app.css';

	let { children } = $props();

	import { setLocale } from '$lib/paraglide/runtime';
	import { language, token } from '$lib/store';
	import { goto } from '$app/navigation';

	language.subscribe((value) => setLocale(value));
	token.subscribe((value) => {
		if (value == '') {
			goto('/login');
		}
	});
</script>

{@render children()}
