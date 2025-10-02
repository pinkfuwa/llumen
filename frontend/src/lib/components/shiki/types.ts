export type ShikiWorkerRequest = {
	code: string;
	lang: string;
	theme: 'light' | 'dark';
};

export type ShikiWorkerResponse = {
	html?: string;
	error?: string;
};
