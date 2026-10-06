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
		SvelteKitPWA({
			base: '/',
			registerType: 'prompt',
			workbox: {
				navigateFallback: null,
				// The SPA has no prerendered pages when Workbox runs.
				globPatterns: ['client/**/*.{js,css,ico,png,svg,webp,webmanifest}'],
				modifyURLPrefix: { 'client/': '' }
			}
		})
	],
	build: {
		// Lightning CSS rejects valid ::highlight() selectors (upstream #1300).
		cssMinify: 'esbuild',
		sourcemap: process.env.NOMAP !== 'T'
	},
	worker: {
		format: 'es'
	},
	server: {
		allowedHosts: ['.trycloudflare.com'],
		proxy: {
			'/api': {
				target: 'http://localhost:8001',
				changeOrigin: true
			}
		}
	}
});
