<script lang="ts">
	import '../app.css';

	let { children } = $props();
	interface Preference {
		theme: 'light' | 'dark';
		language: 'en' | 'zh-tw';
	}

	import { storable } from '$lib/LocalStorage.svelte';
	import { setLocale } from '$lib/paraglide/runtime';
	let preference = storable({ theme: 'light', language: 'en' } as Preference);

	setLocale($preference.language);
</script>

{@render children()}
