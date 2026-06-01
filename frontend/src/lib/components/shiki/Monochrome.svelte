<script>
	import { getThemeName, getThemeStyle } from './shiki';
	import { theme } from '$lib/preference';
	import { derived } from 'svelte/store';

	let { text = '' } = $props();

	let themeName = derived(theme, (x) => getThemeName(x.dark));
	let themeStyle = derived(theme, (x) => getThemeStyle(x.dark));
</script>

<pre class="shiki {$themeName}" style={$themeStyle}><code
		>{#each text.split('\n') as line}<div class="line min-h-6"><span>{line}</span
				></div>{/each}</code
	></pre>
