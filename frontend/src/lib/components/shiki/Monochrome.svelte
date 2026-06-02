<script>
	import { getThemeName, getThemeStyle } from './shiki';
	import { preference } from '$lib/preference/index.svelte';

	let { text = '' } = $props();

	let themeName = $derived(getThemeName(preference.value.theme));
	let themeStyle = $derived(getThemeStyle(preference.value.theme));
</script>

<pre class="shiki {themeName}" style={themeStyle}><code
		>{#each text.split('\n') as line}<div class="line min-h-6"><span>{line}</span
				></div>{/each}</code
	></pre>
