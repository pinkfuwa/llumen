export type WorkerRequest = string;

export type WorkerPayload = ReturnType<typeof import('marked').lexer>;

export type WorkerResponse = {
	input: string;
	data: WorkerPayload;
};
