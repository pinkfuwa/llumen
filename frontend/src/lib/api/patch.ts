import type { MessagePaginateRespChunkKind } from './types';

export interface PartialMessagePaginateRespChunk {
	kind: MessagePaginateRespChunkKind;
	opened?: boolean;
}
