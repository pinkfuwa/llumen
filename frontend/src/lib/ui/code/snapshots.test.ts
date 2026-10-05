import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { CodeSnapshots } from './snapshots';
import { HighlightQueue } from './queue';
import type { CodePaint } from './paint';
import type { CodeToken, HighlightRequest } from './types';

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

function setup() {
	const requests: HighlightRequest[] = [];
	const queue = new HighlightQueue((request) => requests.push(request));
	const paints: CodePaint[] = [];
	const events: string[] = [];
	const snapshots = new CodeSnapshots(queue, () => {
		const index = paints.length;
		const paint = { clear: vi.fn(() => events.push(`clear:${index}`)), refresh: vi.fn() };
		paints.push(paint);
		events.push(`paint:${index}`);
		return paint;
	});
	const update = (text: string, incremental = true, lang = 'js') =>
		snapshots.update({ text, incremental, lang });
	const finish = (index: number, tokens: CodeToken[] = []) =>
		queue.finish({ id: requests[index].id, tokens });
	return { requests, paints, events, snapshots, update, finish };
}

describe('streaming syntax snapshots', () => {
	it('requests the first snapshot immediately and coalesces updates at the next interval', () => {
		const { requests, update, finish } = setup();
		update('const a');
		expect(requests.map((request) => request.text)).toEqual(['const a']);
		finish(0);
		update('const answer');
		vi.advanceTimersByTime(50);
		update('const answer = 42;');
		vi.advanceTimersByTime(49);
		expect(requests).toHaveLength(1);
		vi.advanceTimersByTime(1);
		expect(requests.map((request) => request.text)).toEqual(['const a', 'const answer = 42;']);
	});

	it('keeps painted prefixes through updates and installs new colors before clearing old ones', () => {
		const { paints, events, update, finish } = setup();
		update('const a');
		finish(0);
		update('const answer');
		update('const answer = 42;');
		expect(paints[0].refresh).toHaveBeenCalledTimes(2);
		expect(paints[0].clear).not.toHaveBeenCalled();
		vi.advanceTimersByTime(100);
		expect(paints[0].clear).not.toHaveBeenCalled();
		finish(1);
		expect(events).toEqual(['paint:0', 'paint:1', 'clear:0']);
	});

	it('accepts the first highlight while newer appended text waits for a slow worker', () => {
		const { requests, paints, update, finish } = setup();
		update('const a');
		update('const answer = 42;');
		vi.advanceTimersByTime(100);
		expect(requests).toHaveLength(1);
		finish(0);
		expect(paints).toHaveLength(1);
		expect(requests[1].text).toBe('const answer = 42;');
		expect(paints[0].clear).not.toHaveBeenCalled();
		finish(1);
		expect(paints).toHaveLength(2);
	});

	it('flushes a closed block immediately while retaining its existing colors', () => {
		const { requests, paints, snapshots, update, finish } = setup();
		update('const a');
		finish(0);
		update('const answer = 42;');
		update('const answer = 42;', false);
		expect(requests[1].text).toBe('const answer = 42;');
		expect(paints[0].clear).not.toHaveBeenCalled();
		finish(1);
		vi.advanceTimersByTime(200);
		expect(requests).toHaveLength(2);
		snapshots.destroy();
		expect(paints[1].clear).toHaveBeenCalledOnce();
	});

	it('rejects responses for replaced source and clears invalid colors on language changes', () => {
		const { requests, paints, update, finish } = setup();
		update('const old');
		update('let replacement');
		finish(0);
		expect(paints).toHaveLength(0);
		expect(requests[1].text).toBe('let replacement');
		finish(1);
		update('let replacement', true, 'toml');
		expect(paints[0].clear).toHaveBeenCalledOnce();
		expect(requests[2].language).toBe('toml');
	});

	it('cancels pending intervals and worker callbacks when the component is removed', () => {
		const { requests, paints, snapshots, update, finish } = setup();
		update('const a');
		update('const answer');
		snapshots.destroy();
		vi.advanceTimersByTime(200);
		finish(0);
		expect(requests).toHaveLength(1);
		expect(paints).toHaveLength(0);
	});
});
