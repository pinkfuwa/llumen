import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';

export default defineConfig({
	plugins: [
		/** @type {any} */ (tailwindcss()),
		sveltekit(),
		SvelteKitPWA({
			base: '/',
			registerType: 'prompt'
		})
	],
	build: {
		sourcemap: process.env.NOMAP !== 'T'
	},
	worker: {
		format: 'es'
	},
	esbuild: {
		legalComments: 'external'
	},
	server: {
		allowedHosts: ['.trycloudflare.com'] // allows example.com, foo.example.com, etc.
	}
});
