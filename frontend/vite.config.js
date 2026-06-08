import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import precompileIntl from 'svelte-intl-precompile/sveltekit-plugin';

export default defineConfig({
	plugins: [
		precompileIntl('src/lib/i18n/locales'),
		tailwindcss(),
		sveltekit(),
		SvelteKitPWA({ base: '/', registerType: 'prompt' })
	],
	build: {
		sourcemap: process.env.NOMAP !== 'T'
	},
	worker: {
		format: 'es'
	},
	server: {
		allowedHosts: ['.trycloudflare.com'] // allows example.com, foo.example.com, etc.
	}
});
