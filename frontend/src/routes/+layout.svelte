<script lang="ts">
	import '../app.css';
	import { setLocale } from '$lib/paraglide/runtime';
	import { useLanguage, useTheme } from '$lib/store';
	import { getThemeStyle } from '$lib/theme';

	let { children } = $props();

	let language = useLanguage();
	$effect(() => {
		setLocale($language);
	});

	let theme = useTheme();
</script>

<div class="h-full w-full bg-light text-dark" style={getThemeStyle($theme)}>
	{@render children()}
</div>
