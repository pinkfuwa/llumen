import { codegen } from 'shiki-codegen';
import { writeFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const commonLanguages = [
	'abap',
	'astro',
	'c',
	'c#',
	'cpp',
	'css',
	'dart',
	'dockerfile',
	'elixir',
	'elm',
	'erb',
	'erlang',
	'fortran-free-form',
	'fsharp',
	'gdscript',
	'go',
	'graphql',
	'groovy',
	'handlebars',
	'haskell',
	'html',
	'ini',
	'java',
	'javascript',
	'json',
	'jsx',
	'julia',
	'kotlin',
	'latex',
	'less',
	'lua',
	'makefile',
	'markdown',
	'matlab',
	'nix',
	'objective-c',
	'perl',
	'php',
	'powershell',
	'proto',
	'python',
	'r',
	'racket',
	'ruby',
	'rust',
	'sass',
	'scala',
	'scss',
	'sql',
	'svelte',
	'swift',
	'toml',
	'tsx',
	'typescript',
	'verilog',
	'vue',
	'wasm',
	'wgsl',
	'xml',
	'yaml'
];

let { code } = await codegen({
	langs: commonLanguages,
	themes: ['github-light', 'github-dark'],
	engine: 'javascript',
	typescript: true
});

code = code.replaceAll('@shikijs', 'shiki');

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

await writeFile(
	join(__dirname, 'src', 'lib', 'components', 'shiki', 'shiki.bundle.ts'),
	code,
	'utf8'
);
