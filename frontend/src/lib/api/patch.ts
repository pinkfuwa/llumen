import type { AssistantChunk } from './types';

export type PartialAssistantChunk = AssistantChunk & {
	opened?: boolean;
};
