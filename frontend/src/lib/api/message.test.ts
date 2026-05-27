import { describe, it, expect } from 'vitest';

type Message = {
	id: number;
	stream?: boolean;
	inner: { t: string; c: unknown };
};

// ---------------------------------------------------------------------------
// Algorithms under test (copied from message.svelte.ts to test in isolation)
// ---------------------------------------------------------------------------

/** Push a message into a DESCENDING-sorted array (highest id first).
 *  If same id exists, replace it; otherwise insert at correct position. */
function pushMessage(messages: Message[], m: Message) {
	const idx = messages.findIndex((x) => x.id <= m.id);
	if (idx === -1) {
		messages.push(m);
	} else {
		const sameId = messages[idx].id === m.id;
		messages.splice(idx, Number(sameId), m);
	}
}

/**
 * Remove all messages with id >= msgId from a DESCENDING-sorted array.
 * This is the splice-based cleanup used in updateMessage() and deleteMessage().
 */
function cleanupMessages(messages: Message[], msgId: number) {
	const firstKeepIdx = messages.findIndex((x) => x.id < msgId);
	if (firstKeepIdx === -1) {
		messages.splice(0);
	} else {
		messages.splice(0, firstKeepIdx);
	}
}

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

	it('replaces message even when always streaming at index', () => {
		// Simulate streaming: assistant(id=200) is at index 0
		// delta token update arrives with a new token, but same msg id
		const msgs: Message[] = [
			{
				id: 200,
				stream: true,
				inner: { t: 'assistant', c: 'hello' }
			}
		];
		pushMessage(msgs, {
			id: 200,
			stream: true,
			inner: { t: 'assistant', c: 'hello world' }
		});
		expect(msgs).toHaveLength(1);
		expect(msgs[0].inner.c).toBe('hello world');
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
// cleanupMessages — shared splice logic for updateMessage / deleteMessage
// ---------------------------------------------------------------------------

describe('cleanupMessages (remove id >= msgId)', () => {
	it('bug scenario: halt mid-completion then edit — stale assistant removed', () => {
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

	it('deletes a message at end of array: removes it and all higher ids', () => {
		const msgs: Message[] = [
			{ id: 200, inner: { t: 'assistant', c: [] } },
			{ id: 100, inner: { t: 'user', c: [] } },
			{ id: 50, inner: { t: 'assistant', c: [] } }
		];
		cleanupMessages(msgs, 50);
		expect(msgs).toEqual([]);
	});

	it('deletes middle message: removes it and all higher ids', () => {
		const msgs: Message[] = [
			{ id: 200, inner: { t: 'assistant', c: [] } },
			{ id: 150, inner: { t: 'user', c: [] } },
			{ id: 100, inner: { t: 'assistant', c: [] } }
		];
		cleanupMessages(msgs, 150);
		expect(msgs).toEqual([{ id: 100, inner: { t: 'assistant', c: [] } }]);
	});

	it('empty array is a no-op', () => {
		const msgs: Message[] = [];
		cleanupMessages(msgs, 100);
		expect(msgs).toEqual([]);
	});

	it('splice(0) clears correctly when firstKeepIdx === -1', () => {
		const msgs: Message[] = [
			{ id: 200, inner: { t: 'assistant', c: [] } },
			{ id: 150, inner: { t: 'user', c: [] } }
		];
		cleanupMessages(msgs, 100);
		expect(msgs).toEqual([]);
	});
});

// ---------------------------------------------------------------------------
// Integration: pushMessage + cleanup ordering invariant
// ---------------------------------------------------------------------------

describe('invariants through push + cleanup cycle', () => {
	it('after push then cleanup, remaining messages preserve descending order', () => {
		const msgs: Message[] = [];
		// Build a multi-turn conversation
		pushMessage(msgs, { id: 50, inner: { t: 'user', c: 'q1' } });
		pushMessage(msgs, { id: 75, inner: { t: 'assistant', c: 'a1' } });
		pushMessage(msgs, { id: 100, inner: { t: 'user', c: 'q2' } });
		pushMessage(msgs, { id: 150, inner: { t: 'assistant', c: 'a2' } });
		pushMessage(msgs, { id: 200, inner: { t: 'user', c: 'q3' } });
		pushMessage(msgs, { id: 250, inner: { t: 'assistant', c: 'a3' } });

		expect(msgs.map((m) => m.id)).toEqual([250, 200, 150, 100, 75, 50]);

		// Edit q2 (id=100): backend deletes id >= 100
		// Should keep only q1 (50) and a1 (75)
		cleanupMessages(msgs, 100);

		expect(msgs.map((m) => m.id)).toEqual([75, 50]);

		// Verify descending order preserved
		expect(msgs[0].inner.t).toBe('assistant');
		expect(msgs[1].inner.t).toBe('user');
	});

	it('after cleanup, new messages can be pushed and maintain order', () => {
		const msgs: Message[] = [
			{ id: 300, inner: { t: 'assistant', c: [] } },
			{ id: 200, inner: { t: 'user', c: [] } },
			{ id: 100, inner: { t: 'assistant', c: [] } }
		];

		// Delete/update msgId=200
		cleanupMessages(msgs, 200);

		expect(msgs.map((m) => m.id)).toEqual([100]);

		// Simulate backend creating new messages
		pushMessage(msgs, { id: 400, inner: { t: 'user', c: [] } });
		pushMessage(msgs, { id: 500, inner: { t: 'assistant', c: [] } });

		expect(msgs.map((m) => m.id)).toEqual([500, 400, 100]);
		expect(msgs.map((m) => m.inner.t)).toEqual(['assistant', 'user', 'assistant']);
	});
});
