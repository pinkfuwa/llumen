import type { CodeLanguage } from './languages';
import type { CodeToken, HighlightRequest, HighlightResponse } from './types';

type Task = { owner: object; request: HighlightRequest; complete?: (tokens: CodeToken[]) => void };

/** Keeps one worker request in flight and only the latest pending snapshot per owner. */
export class HighlightQueue {
	private pending = new Map<object, Task>();
	private active?: Task;
	private nextId = 0;
	private failed = false;

	constructor(private send: (request: HighlightRequest) => void) {}

	request(
		owner: object,
		text: string,
		language: CodeLanguage,
		complete: (tokens: CodeToken[]) => void
	) {
		if (this.failed) return;
		this.pending.set(owner, { owner, request: { id: ++this.nextId, text, language }, complete });
		this.pump();
	}

	cancel(owner: object) {
		this.pending.delete(owner);
		if (this.active?.owner === owner) this.active.complete = undefined;
	}

	finish(response: HighlightResponse) {
		if (response.id !== this.active?.request.id) return;
		const task = this.active;
		this.active = undefined;
		try {
			task.complete?.(response.tokens);
		} finally {
			this.pump();
		}
	}

	fail() {
		this.failed = true;
		this.active = undefined;
		this.pending.clear();
	}

	private pump() {
		if (this.active || this.failed) return;
		const task = this.pending.values().next().value;
		if (!task) return;
		this.pending.delete(task.owner);
		this.active = task;
		try {
			this.send(task.request);
		} catch {
			this.fail();
		}
	}
}
