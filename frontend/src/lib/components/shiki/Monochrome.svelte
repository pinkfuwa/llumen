<script>
	import { getThemeName, getThemeStyle } from './shiki';
	import { isLightTheme } from '$lib/preference';

	let { text = '' } = $props();

	let themeName = $derived(getThemeName($isLightTheme));
	let themeStyle = $derived(getThemeStyle($isLightTheme));
</script>

<pre class="shiki {themeName}" style={themeStyle}><code
		>{#each text.split('\n') as line}<div class="line min-h-6"><span>{line}</span
				></div>{/each}</code
	></pre>
