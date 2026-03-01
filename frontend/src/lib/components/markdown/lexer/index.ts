export { lexBlock } from './block';
export { lexInline } from './inline';
export { LexTokenKind } from './types';
export type { LexToken } from './types';

import { lexBlock } from './block';
import type { LexToken } from './types';

export function lex(source: string): LexToken[] {
	return lexBlock(source);
}
