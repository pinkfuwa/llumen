<script lang="ts">
	import '../app.css';
	import { useLanguage, useTheme } from '$lib/store';
	import { setTheme } from '$lib/theme';
	import { setLocale } from '$lib/i18n';

	let { children } = $props();

	let language = useLanguage();
	let theme = useTheme();

	$effect(() => {
		setLocale($language);
		setTheme($theme);
	});
</script>

<div class="h-full w-full bg-light text-dark">
	{@render children()}
</div>
