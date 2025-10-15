/* Generate by shiki/codegen */
import type {
	DynamicImportLanguageRegistration,
	DynamicImportThemeRegistration,
	HighlighterGeneric
} from 'shiki/types';
import { createSingletonShorthands, createdBundledHighlighter } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine-javascript.mjs';

type BundledLanguage =
	| 'abap'
	| 'astro'
	| 'c'
	| 'csharp'
	| 'c#'
	| 'cs'
	| 'cpp'
	| 'c++'
	| 'css'
	| 'dart'
	| 'docker'
	| 'dockerfile'
	| 'elixir'
	| 'elm'
	| 'erb'
	| 'erlang'
	| 'erl'
	| 'fortran-free-form'
	| 'f90'
	| 'f95'
	| 'f03'
	| 'f08'
	| 'f18'
	| 'fsharp'
	| 'f#'
	| 'fs'
	| 'gdscript'
	| 'go'
	| 'graphql'
	| 'gql'
	| 'groovy'
	| 'handlebars'
	| 'hbs'
	| 'haskell'
	| 'hs'
	| 'html'
	| 'ini'
	| 'properties'
	| 'java'
	| 'javascript'
	| 'js'
	| 'json'
	| 'jsx'
	| 'julia'
	| 'jl'
	| 'kotlin'
	| 'kt'
	| 'kts'
	| 'latex'
	| 'less'
	| 'lua'
	| 'make'
	| 'makefile'
	| 'markdown'
	| 'md'
	| 'matlab'
	| 'nix'
	| 'objective-c'
	| 'objc'
	| 'perl'
	| 'php'
	| 'powershell'
	| 'ps'
	| 'ps1'
	| 'proto'
	| 'protobuf'
	| 'python'
	| 'py'
	| 'r'
	| 'racket'
	| 'ruby'
	| 'rb'
	| 'rust'
	| 'rs'
	| 'sass'
	| 'scala'
	| 'scss'
	| 'sql'
	| 'svelte'
	| 'swift'
	| 'toml'
	| 'tsx'
	| 'typescript'
	| 'ts'
	| 'verilog'
	| 'vue'
	| 'wasm'
	| 'wgsl'
	| 'xml'
	| 'yaml'
	| 'yml'
	| 'bash'
	| 'asm';
type BundledTheme = 'github-light' | 'github-dark';
type Highlighter = HighlighterGeneric<BundledLanguage, BundledTheme>;

const bundledLanguages = {
	abap: () => import('shiki/langs/abap.mjs'),
	astro: () => import('shiki/langs/astro.mjs'),
	c: () => import('shiki/langs/c.mjs'),
	csharp: () => import('shiki/langs/csharp.mjs'),
	'c#': () => import('shiki/langs/csharp.mjs'),
	cs: () => import('shiki/langs/csharp.mjs'),
	cpp: () => import('shiki/langs/cpp.mjs'),
	'c++': () => import('shiki/langs/cpp.mjs'),
	css: () => import('shiki/langs/css.mjs'),
	dart: () => import('shiki/langs/dart.mjs'),
	docker: () => import('shiki/langs/docker.mjs'),
	dockerfile: () => import('shiki/langs/docker.mjs'),
	elixir: () => import('shiki/langs/elixir.mjs'),
	elm: () => import('shiki/langs/elm.mjs'),
	erb: () => import('shiki/langs/erb.mjs'),
	erlang: () => import('shiki/langs/erlang.mjs'),
	erl: () => import('shiki/langs/erlang.mjs'),
	'fortran-free-form': () => import('shiki/langs/fortran-free-form.mjs'),
	f90: () => import('shiki/langs/fortran-free-form.mjs'),
	f95: () => import('shiki/langs/fortran-free-form.mjs'),
	f03: () => import('shiki/langs/fortran-free-form.mjs'),
	f08: () => import('shiki/langs/fortran-free-form.mjs'),
	f18: () => import('shiki/langs/fortran-free-form.mjs'),
	fsharp: () => import('shiki/langs/fsharp.mjs'),
	'f#': () => import('shiki/langs/fsharp.mjs'),
	fs: () => import('shiki/langs/fsharp.mjs'),
	gdscript: () => import('shiki/langs/gdscript.mjs'),
	go: () => import('shiki/langs/go.mjs'),
	graphql: () => import('shiki/langs/graphql.mjs'),
	gql: () => import('shiki/langs/graphql.mjs'),
	groovy: () => import('shiki/langs/groovy.mjs'),
	handlebars: () => import('shiki/langs/handlebars.mjs'),
	hbs: () => import('shiki/langs/handlebars.mjs'),
	haskell: () => import('shiki/langs/haskell.mjs'),
	hs: () => import('shiki/langs/haskell.mjs'),
	html: () => import('shiki/langs/html.mjs'),
	ini: () => import('shiki/langs/ini.mjs'),
	properties: () => import('shiki/langs/ini.mjs'),
	java: () => import('shiki/langs/java.mjs'),
	javascript: () => import('shiki/langs/javascript.mjs'),
	js: () => import('shiki/langs/javascript.mjs'),
	json: () => import('shiki/langs/json.mjs'),
	jsx: () => import('shiki/langs/jsx.mjs'),
	julia: () => import('shiki/langs/julia.mjs'),
	jl: () => import('shiki/langs/julia.mjs'),
	kotlin: () => import('shiki/langs/kotlin.mjs'),
	kt: () => import('shiki/langs/kotlin.mjs'),
	kts: () => import('shiki/langs/kotlin.mjs'),
	latex: () => import('shiki/langs/latex.mjs'),
	less: () => import('shiki/langs/less.mjs'),
	lua: () => import('shiki/langs/lua.mjs'),
	make: () => import('shiki/langs/make.mjs'),
	makefile: () => import('shiki/langs/make.mjs'),
	markdown: () => import('shiki/langs/markdown.mjs'),
	md: () => import('shiki/langs/markdown.mjs'),
	matlab: () => import('shiki/langs/matlab.mjs'),
	nix: () => import('shiki/langs/nix.mjs'),
	'objective-c': () => import('shiki/langs/objective-c.mjs'),
	objc: () => import('shiki/langs/objective-c.mjs'),
	perl: () => import('shiki/langs/perl.mjs'),
	php: () => import('shiki/langs/php.mjs'),
	powershell: () => import('shiki/langs/powershell.mjs'),
	ps: () => import('shiki/langs/powershell.mjs'),
	ps1: () => import('shiki/langs/powershell.mjs'),
	proto: () => import('shiki/langs/proto.mjs'),
	protobuf: () => import('shiki/langs/proto.mjs'),
	python: () => import('shiki/langs/python.mjs'),
	py: () => import('shiki/langs/python.mjs'),
	r: () => import('shiki/langs/r.mjs'),
	racket: () => import('shiki/langs/racket.mjs'),
	ruby: () => import('shiki/langs/ruby.mjs'),
	rb: () => import('shiki/langs/ruby.mjs'),
	rust: () => import('shiki/langs/rust.mjs'),
	rs: () => import('shiki/langs/rust.mjs'),
	sass: () => import('shiki/langs/sass.mjs'),
	scala: () => import('shiki/langs/scala.mjs'),
	scss: () => import('shiki/langs/scss.mjs'),
	sql: () => import('shiki/langs/sql.mjs'),
	svelte: () => import('shiki/langs/svelte.mjs'),
	swift: () => import('shiki/langs/swift.mjs'),
	toml: () => import('shiki/langs/toml.mjs'),
	tsx: () => import('shiki/langs/tsx.mjs'),
	typescript: () => import('shiki/langs/typescript.mjs'),
	ts: () => import('shiki/langs/typescript.mjs'),
	verilog: () => import('shiki/langs/verilog.mjs'),
	vue: () => import('shiki/langs/vue.mjs'),
	wasm: () => import('shiki/langs/wasm.mjs'),
	wgsl: () => import('shiki/langs/wgsl.mjs'),
	xml: () => import('shiki/langs/xml.mjs'),
	yaml: () => import('shiki/langs/yaml.mjs'),
	yml: () => import('shiki/langs/yaml.mjs'),
	bash: () => import('shiki/langs/bash.mjs'),
	shell: () => import('shiki/langs/shell.mjs'),
	asm: () => import('shiki/langs/asm.mjs')
} as Record<BundledLanguage, DynamicImportLanguageRegistration>;

const bundledThemes = {
	'github-light': () => import('shiki/themes/github-light.mjs'),
	'github-dark': () => import('shiki/themes/github-dark.mjs')
} as Record<BundledTheme, DynamicImportThemeRegistration>;

const createHighlighter = /* @__PURE__ */ createdBundledHighlighter<BundledLanguage, BundledTheme>({
	langs: bundledLanguages,
	themes: bundledThemes,
	engine: () => createJavaScriptRegexEngine()
});

const {
	codeToHtml,
	codeToHast,
	codeToTokensBase,
	codeToTokens,
	codeToTokensWithThemes,
	getSingletonHighlighter,
	getLastGrammarState
} = /* @__PURE__ */ createSingletonShorthands<BundledLanguage, BundledTheme>(createHighlighter);

export {
	bundledLanguages,
	bundledThemes,
	codeToHast,
	codeToHtml,
	codeToTokens,
	codeToTokensBase,
	codeToTokensWithThemes,
	createHighlighter,
	getLastGrammarState,
	getSingletonHighlighter
};
export type { BundledLanguage, BundledTheme, Highlighter };
