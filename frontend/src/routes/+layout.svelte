<script lang="ts">
	import '../app.css';

	let { children } = $props();

	import { setLocale } from '$lib/paraglide/runtime';
	import { language } from '$lib/localStorage';

	setLocale(language().current);
</script>

{@render children()}
