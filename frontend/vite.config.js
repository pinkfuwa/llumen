import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import license from 'rollup-plugin-license';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	build: {
		sourcemap: process.env.NOMAP != 'T',
		rollupOptions: {
			plugins: [
				license({
					thirdParty: {
						includePrivate: false,
						output: {
							file: path.join(__dirname, '.svelte-kit', 'output', 'client', 'LICENSE'),
							template() {
								return `llumen - Third-Party Licenses

This software uses third-party libraries and dependencies.

Complete license information for all dependencies can be found in the source repository:

Frontend Dependencies:
  https://github.com/eason/llumen/blob/main/frontend/THIRDPARTY.txt

Backend Dependencies:
  https://github.com/eason/llumen/blob/main/backend/THIRDPARTY.toml

For the full source code and additional information, visit:
  https://github.com/eason/llumen

This project is licensed under the Mozilla Public License 2.0 (MPL-2.0).
See the LICENSE file in the repository root for details.
`;
							}
						}
					}
				})
			]
		}
	},
	worker: {
		format: 'es'
	},
	esbuild: {
		legalComments: 'external'
	}
});
