export type WorkerRequest = string;

export type WorkerResponse = ReturnType<typeof import('marked').lexer>;
