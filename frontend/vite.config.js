import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import { paraglideVitePlugin } from '@inlang/paraglide-js';

export default defineConfig({
	plugins: [
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide',
			strategy: ['localStorage', 'baseLocale']
		}),
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
