<script lang="ts">
	import '../app.css';

	let { children } = $props();
	interface Preference {
		theme: 'light' | 'dark';
		language: 'en' | 'zh-tw';
	}

	import { storable } from '$lib/LocalStorage';
	import { setLocale } from '$lib/paraglide/runtime';
	import { defaultPreference } from '$lib/perference';

	let preference = storable(defaultPreference);

	setLocale($preference.language);
</script>

{@render children()}
