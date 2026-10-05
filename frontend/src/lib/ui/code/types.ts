import type { ShjToken } from '@speed-highlight/core/tokenize';
import type { CodeLanguage } from './languages';

export type CodeToken = { start: number; end: number; type: ShjToken };
export type HighlightRequest = { id: number; text: string; language: CodeLanguage };
export type HighlightResponse = { id: number; tokens: CodeToken[] };
export type CodeSnapshot = { text: string; lang: string; incremental: boolean };
