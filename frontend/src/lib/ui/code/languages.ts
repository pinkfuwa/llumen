const names = [
	'asm',
	'bash',
	'bf',
	'c',
	'css',
	'csv',
	'diff',
	'docker',
	'git',
	'go',
	'html',
	'http',
	'ini',
	'java',
	'js',
	'jsdoc',
	'json',
	'leanpub-md',
	'log',
	'lua',
	'make',
	'md',
	'pl',
	'plain',
	'py',
	'regex',
	'rs',
	'sql',
	'todo',
	'toml',
	'ts',
	'uri',
	'xml',
	'yaml'
] as const;

export type CodeLanguage = (typeof names)[number];

const aliases: Record<string, CodeLanguage> = {
	javascript: 'js',
	typescript: 'ts',
	python: 'py',
	rust: 'rs',
	perl: 'pl',
	markdown: 'md',
	shellscript: 'bash',
	shell: 'bash',
	sh: 'bash',
	zsh: 'bash',
	dockerfile: 'docker',
	golang: 'go',
	yml: 'yaml',
	htm: 'html',
	svg: 'xml',
	makefile: 'make',
	regexp: 'regex',
	url: 'uri',
	text: 'plain',
	txt: 'plain',
	plaintext: 'plain'
};

export function codeLanguage(language: string): CodeLanguage | undefined {
	const name = language.trim().toLowerCase();
	if (Object.hasOwn(aliases, name)) return aliases[name];
	return names.find((candidate) => candidate === name);
}
