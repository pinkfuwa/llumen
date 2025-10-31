export type WorkerRequest = string;

export type WorkerToken = {
	type: string;
	raw: string;
	text?: string;
	displayMode?: boolean;
	[key: string]: any;
};

export type WorkerResponse = WorkerToken[];
