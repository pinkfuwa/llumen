import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';
import path from 'path';

export default defineConfig({
	plugins: [sveltekit()],
	resolve: {
		alias: {
			$lib: path.resolve('src/lib')
		}
	},
	test: {
		alias: {
			'$app/environment': path.resolve('src/test/environment.ts')
		},
		globals: true,
		environment: 'jsdom',
		include: ['src/**/*.{test,spec}.{js,ts}'],
		coverage: {
			provider: 'v8',
			include: ['src/lib/**/*.{js,ts}'],
			exclude: [
				'**/*.{test,spec}.{js,ts}',
				'**/*.d.ts',
				'**/tests/**',
				'src/lib/api/types.ts',
				'src/lib/i18n/generated/**',
				'src/lib/components/shiki/shiki.bundle.ts'
			],
			reporter: ['text-summary', 'json-summary', 'html']
		}
	}
});
