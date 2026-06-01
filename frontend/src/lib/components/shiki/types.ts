export type ShikiWorkerRequest = {
	code: string;
	lang: string;
	dark: boolean;
};

export type ShikiWorkerResponse = {
	html?: string;
	error?: string;
};
