import type { BundledTheme } from './shiki.bundle';

export type ShikiWorkerRequest = {
	code: string;
	lang: string;
	theme: BundledTheme;
};

export type ShikiWorkerResponse = {
	html?: string;
	error?: string;
};
