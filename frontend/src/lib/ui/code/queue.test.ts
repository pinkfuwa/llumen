import { describe, expect, it } from 'vitest';
import { HighlightQueue } from './queue';
import type { HighlightRequest } from './types';

describe('syntax snapshot queue', () => {
	it('coalesces streaming snapshots while letting other blocks progress', () => {
		const sent: HighlightRequest[] = [];
		const results: string[] = [];
		const queue = new HighlightQueue((request) => sent.push(request));
		const first = {},
			second = {};
		queue.request(first, 'one', 'js', () => results.push('one'));
		queue.request(first, 'two', 'js', () => results.push('two'));
		queue.request(second, 'other', 'toml', () => results.push('other'));
		queue.request(first, 'three', 'js', () => results.push('three'));
		expect(sent.map((request) => request.text)).toEqual(['one']);
		queue.finish({ id: sent[0].id, tokens: [] });
		expect(sent.map((request) => request.text)).toEqual(['one', 'three']);
		queue.finish({ id: sent[1].id, tokens: [] });
		expect(sent.map((request) => request.text)).toEqual(['one', 'three', 'other']);
		queue.finish({ id: sent[2].id, tokens: [] });
		expect(results).toEqual(['one', 'three', 'other']);
	});

	it('cancels queued and active callbacks without releasing the worker early', () => {
		const sent: HighlightRequest[] = [];
		const queue = new HighlightQueue((request) => sent.push(request));
		const owner = {};
		let completed = false;
		queue.request(owner, 'old', 'js', () => {
			completed = true;
		});
		queue.request(owner, 'pending', 'js', () => {
			completed = true;
		});
		queue.cancel(owner);
		queue.request({}, 'live', 'json', () => {});
		queue.finish({ id: -1, tokens: [] });
		expect(sent).toHaveLength(1);
		queue.finish({ id: sent[0].id, tokens: [] });
		expect(sent.map((request) => request.text)).toEqual(['old', 'live']);
		expect(completed).toBe(false);
	});

	it('clears pending source snapshots when worker startup or execution fails', () => {
		const queue = new HighlightQueue(() => {
			throw new Error('blocked worker');
		});
		expect(() => queue.request({}, 'source', 'js', () => {})).not.toThrow();
		expect(() => queue.request({}, 'new source', 'js', () => {})).not.toThrow();
	});
});
