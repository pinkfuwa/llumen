import { describe, it, expect } from 'vitest';

type Message = {
	id: number;
	stream?: boolean;
	inner: { t: string; c: unknown };
};

// ---------------------------------------------------------------------------
// Algorithms under test (copied from message.svelte.ts to test in isolation)
// ---------------------------------------------------------------------------

function firstLeIdx(arr: Message[], target: number): number {
	let lo = 0,
		hi = arr.length;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		if (arr[mid].id <= target) hi = mid;
		else lo = mid + 1;
	}
	return lo;
}

function firstLtIdx(arr: Message[], target: number): number {
	let lo = 0,
		hi = arr.length;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		if (arr[mid].id < target) hi = mid;
		else lo = mid + 1;
	}
	return lo;
}

function pushMessage(messages: Message[], m: Message) {
	const idx = firstLeIdx(messages, m.id);
	if (idx === messages.length) messages.push(m);
	else {
		const sameId = messages[idx].id === m.id;
		messages.splice(idx, Number(sameId), m);
	}
}

function cleanupMessages(messages: Message[], msgId: number) {
	const firstKeepIdx = firstLtIdx(messages, msgId);
	if (firstKeepIdx === messages.length) messages.splice(0);
	else messages.splice(0, firstKeepIdx);
}

function mergeDescending(a: Message[], b: Message[], streamingMessageId: number | null): Message[] {
	const result: Message[] = [];
	let i = 0,
		j = 0;
	while (i < a.length && j < b.length) {
		const msg = a[i];
		const fet = b[j];
		if (msg.id > fet.id) {
			result.push(msg);
			i++;
		} else if (msg.id < fet.id) {
			result.push(fet);
			j++;
		} else {
			if (msg.id === streamingMessageId) {
				result.push(msg);
				while (j < b.length && b[j].id === msg.id) j++;
			} else {
				result.push(fet);
				j++;
			}
			i++;
		}
	}
	result.push(...a.slice(i), ...b.slice(j));
	return result;
}

describe('firstLeIdx', () => {
	it('returns length for empty array', () => {
		expect(firstLeIdx([], 100)).toBe(0);
	});

	it('returns 0 when target >= first element', () => {
		const arr = [{ id: 100, inner: { t: 'user', c: null } }];
		expect(firstLeIdx(arr, 100)).toBe(0);
		expect(firstLeIdx(arr, 200)).toBe(0);
	});

	it('returns correct index in descending array', () => {
		const arr = [
			{ id: 200, inner: { t: 'user', c: null } },
			{ id: 150, inner: { t: 'user', c: null } },
			{ id: 100, inner: { t: 'user', c: null } },
			{ id: 50, inner: { t: 'user', c: null } }
		];
		expect(firstLeIdx(arr, 150)).toBe(1);
		expect(firstLeIdx(arr, 100)).toBe(2);
		expect(firstLeIdx(arr, 75)).toBe(3);
		expect(firstLeIdx(arr, 49)).toBe(4);
	});
});

describe('firstLtIdx', () => {
	it('returns correct index for < condition', () => {
		const arr = [
			{ id: 200, inner: { t: 'user', c: null } },
			{ id: 100, inner: { t: 'user', c: null } }
		];
		expect(firstLtIdx(arr, 200)).toBe(1);
		expect(firstLtIdx(arr, 100)).toBe(2);
		expect(firstLtIdx(arr, 50)).toBe(2);
	});
});

// ---------------------------------------------------------------------------
// pushMessage
// ---------------------------------------------------------------------------

describe('pushMessage', () => {
	it('inserts into empty array', () => {
		const msgs: Message[] = [];
		pushMessage(msgs, { id: 100, inner: { t: 'user', c: null } });
		expect(msgs).toEqual([{ id: 100, inner: { t: 'user', c: null } }]);
	});

	it('maintains descending order: higher id before lower', () => {
		const msgs: Message[] = [];
		pushMessage(msgs, { id: 100, inner: { t: 'user', c: null } });
		pushMessage(msgs, { id: 200, inner: { t: 'assistant', c: null } });
		expect(msgs).toEqual([
			{ id: 200, inner: { t: 'assistant', c: null } },
			{ id: 100, inner: { t: 'user', c: null } }
		]);
	});

	it('inserts in correct position between existing messages', () => {
		const msgs: Message[] = [
			{ id: 400, inner: { t: 'assistant', c: null } },
			{ id: 200, inner: { t: 'user', c: null } }
		];
		pushMessage(msgs, { id: 300, inner: { t: 'assistant', c: null } });
		expect(msgs).toEqual([
			{ id: 400, inner: { t: 'assistant', c: null } },
			{ id: 300, inner: { t: 'assistant', c: null } },
			{ id: 200, inner: { t: 'user', c: null } }
		]);
	});

	it('replaces message with same id', () => {
		const msgs: Message[] = [
			{ id: 200, inner: { t: 'assistant', c: null } },
			{ id: 100, inner: { t: 'user', c: null } }
		];
		pushMessage(msgs, {
			id: 100,
			stream: true,
			inner: { t: 'user', c: 'updated' }
		});
		expect(msgs).toHaveLength(2);
		expect(msgs[1].id).toBe(100);
		expect(msgs[1].stream).toBe(true);
		expect(msgs[1].inner.c).toBe('updated');
	});

	it('appends at end when new message has lowest id', () => {
		const msgs: Message[] = [{ id: 50, inner: { t: 'assistant', c: null } }];
		pushMessage(msgs, { id: 25, inner: { t: 'user', c: null } });
		expect(msgs).toEqual([
			{ id: 50, inner: { t: 'assistant', c: null } },
			{ id: 25, inner: { t: 'user', c: null } }
		]);
	});
});

// ---------------------------------------------------------------------------
// cleanupMessages
// ---------------------------------------------------------------------------

describe('cleanupMessages (remove id >= msgId)', () => {
	it('halt mid-completion then edit — stale assistant removed', () => {
		const msgs: Message[] = [
			{ id: 200, stream: false, inner: { t: 'assistant', c: [] } },
			{ id: 100, inner: { t: 'user', c: [] } }
		];
		cleanupMessages(msgs, 100);
		expect(msgs).toEqual([]);
	});

	it('normal completion then edit: removes user and assistant', () => {
		const msgs: Message[] = [
			{ id: 200, inner: { t: 'assistant', c: [] } },
			{ id: 100, inner: { t: 'user', c: [] } }
		];
		cleanupMessages(msgs, 100);
		expect(msgs).toEqual([]);
	});

	it('multi-turn: edits a message with later responses — keeps earlier ones', () => {
		const msgs: Message[] = [
			{ id: 400, inner: { t: 'assistant', c: [] } },
			{ id: 350, inner: { t: 'user', c: [] } },
			{ id: 300, inner: { t: 'assistant', c: [] } },
			{ id: 200, inner: { t: 'user', c: [] } }
		];
		cleanupMessages(msgs, 350);
		expect(msgs).toEqual([
			{ id: 300, inner: { t: 'assistant', c: [] } },
			{ id: 200, inner: { t: 'user', c: [] } }
		]);
	});

	it('edits the oldest message: removes all', () => {
		const msgs: Message[] = [
			{ id: 400, inner: { t: 'assistant', c: [] } },
			{ id: 350, inner: { t: 'assistant', c: [] } },
			{ id: 300, inner: { t: 'user', c: [] } }
		];
		cleanupMessages(msgs, 300);
		expect(msgs).toEqual([]);
	});

	it('deletes newest message: removes only it', () => {
		const msgs: Message[] = [
			{ id: 400, inner: { t: 'assistant', c: [] } },
			{ id: 350, inner: { t: 'user', c: [] } }
		];
		cleanupMessages(msgs, 400);
		expect(msgs).toEqual([{ id: 350, inner: { t: 'user', c: [] } }]);
	});

	it('empty array is a no-op', () => {
		const msgs: Message[] = [];
		cleanupMessages(msgs, 100);
		expect(msgs).toEqual([]);
	});
});

// ---------------------------------------------------------------------------
// mergeDescending — syncMessages core algorithm
// ---------------------------------------------------------------------------

describe('mergeDescending (syncMessages)', () => {
	it('deduplicates by taking fetched when same id and not streaming', () => {
		const local: Message[] = [
			{ id: 150, stream: true, inner: { t: 'user', c: 'old' } },
			{ id: 100, inner: { t: 'assistant', c: 'a' } }
		];
		const fetched: Message[] = [
			{ id: 150, inner: { t: 'user', c: 'persisted' } },
			{ id: 100, inner: { t: 'assistant', c: 'a' } }
		];
		const result = mergeDescending(local, fetched, null);
		expect(result).toEqual(fetched);
	});

	it('keeps streaming message over fetched duplicate', () => {
		const local: Message[] = [
			{ id: 200, stream: true, inner: { t: 'assistant', c: 'hello' } },
			{ id: 100, inner: { t: 'user', c: 'hi' } }
		];
		const fetched: Message[] = [
			{ id: 200, inner: { t: 'assistant', c: 'hello' } },
			{ id: 100, inner: { t: 'user', c: 'hi' } }
		];
		const result = mergeDescending(local, fetched, 200);
		expect(result).toHaveLength(2);
		expect(result[0].id).toBe(200);
		expect(result[0].stream).toBe(true);
		expect(result[1].id).toBe(100);
	});

	it('merges items unique to each array', () => {
		const local: Message[] = [
			{ id: 300, inner: { t: 'assistant', c: 'a' } },
			{ id: 100, inner: { t: 'user', c: 'q' } }
		];
		const fetched: Message[] = [
			{ id: 400, inner: { t: 'assistant', c: 'b' } },
			{ id: 200, inner: { t: 'user', c: 'c' } }
		];
		const result = mergeDescending(local, fetched, null);
		expect(result.map((m) => m.id)).toEqual([400, 300, 200, 100]);
	});

	it('handles empty local', () => {
		const fetched: Message[] = [
			{ id: 200, inner: { t: 'user', c: 'x' } },
			{ id: 100, inner: { t: 'user', c: 'y' } }
		];
		const result = mergeDescending([], fetched, null);
		expect(result).toEqual(fetched);
	});

	it('handles empty fetched', () => {
		const local: Message[] = [
			{ id: 200, inner: { t: 'user', c: 'x' } },
			{ id: 100, inner: { t: 'user', c: 'y' } }
		];
		const result = mergeDescending(local, [], null);
		expect(result).toEqual(local);
	});

	it('handles both empty', () => {
		const result = mergeDescending([], [], null);
		expect(result).toEqual([]);
	});

	it('deduplicates when streaming id appears multiple times in fetched', () => {
		const local: Message[] = [
			{ id: 200, stream: true, inner: { t: 'assistant', c: 'live' } },
			{ id: 100, inner: { t: 'user', c: 'q' } }
		];
		const fetched: Message[] = [
			{ id: 200, inner: { t: 'assistant', c: 'live' } },
			// duplicate rows should not happen but merge handles it
			{ id: 200, inner: { t: 'assistant', c: 'live' } },
			{ id: 100, inner: { t: 'user', c: 'q' } }
		];
		const result = mergeDescending(local, fetched, 200);
		expect(result).toHaveLength(2);
		expect(result[0].stream).toBe(true);
	});

	it('interleaved merge: local and fetched have no shared ids', () => {
		const local: Message[] = [
			{ id: 250, inner: { t: 'assistant', c: 'a' } },
			{ id: 150, inner: { t: 'user', c: 'b' } },
			{ id: 50, inner: { t: 'assistant', c: 'c' } }
		];
		const fetched: Message[] = [
			{ id: 300, inner: { t: 'user', c: 'd' } },
			{ id: 200, inner: { t: 'assistant', c: 'e' } },
			{ id: 100, inner: { t: 'user', c: 'f' } }
		];
		const result = mergeDescending(local, fetched, null);
		expect(result.map((m) => m.id)).toEqual([300, 250, 200, 150, 100, 50]);
	});
});

// ---------------------------------------------------------------------------
// Integration: pushMessage + cleanup + merge cycles
// ---------------------------------------------------------------------------

describe('invariants through push + cleanup + merge cycle', () => {
	it('push then cleanup preserves descending order', () => {
		const msgs: Message[] = [];
		pushMessage(msgs, { id: 50, inner: { t: 'user', c: 'q1' } });
		pushMessage(msgs, { id: 75, inner: { t: 'assistant', c: 'a1' } });
		pushMessage(msgs, { id: 100, inner: { t: 'user', c: 'q2' } });
		pushMessage(msgs, { id: 150, inner: { t: 'assistant', c: 'a2' } });
		pushMessage(msgs, { id: 200, inner: { t: 'user', c: 'q3' } });
		pushMessage(msgs, { id: 250, inner: { t: 'assistant', c: 'a3' } });

		expect(msgs.map((m) => m.id)).toEqual([250, 200, 150, 100, 75, 50]);

		cleanupMessages(msgs, 100);
		expect(msgs.map((m) => m.id)).toEqual([75, 50]);
	});

	it('after cleanup, new messages can be pushed and maintain order', () => {
		const msgs: Message[] = [
			{ id: 300, inner: { t: 'assistant', c: [] } },
			{ id: 200, inner: { t: 'user', c: [] } },
			{ id: 100, inner: { t: 'assistant', c: [] } }
		];
		cleanupMessages(msgs, 200);
		expect(msgs.map((m) => m.id)).toEqual([100]);

		pushMessage(msgs, { id: 400, inner: { t: 'user', c: [] } });
		pushMessage(msgs, { id: 500, inner: { t: 'assistant', c: [] } });
		expect(msgs.map((m) => m.id)).toEqual([500, 400, 100]);
	});

	it('merge after push+cleanup cycle', () => {
		const msgs: Message[] = [];
		pushMessage(msgs, { id: 100, inner: { t: 'user', c: 'q1' } });
		pushMessage(msgs, { id: 200, inner: { t: 'assistant', c: 'a1' } });

		// Edit: delete messages >= 100
		cleanupMessages(msgs, 100);
		expect(msgs).toEqual([]);

		// New messages created via merge (fetched is descending per backend order_by_desc)
		const fetched: Message[] = [
			{ id: 400, inner: { t: 'assistant', c: 'a2' } },
			{ id: 300, inner: { t: 'user', c: 'q2' } }
		];
		const result = mergeDescending(msgs, fetched, null);
		expect(result.map((m) => m.id)).toEqual([400, 300]);
	});
});
