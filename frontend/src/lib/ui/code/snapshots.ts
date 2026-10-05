import { codeLanguage, type CodeLanguage } from './languages';
import type { CodePaint } from './paint';
import type { HighlightQueue } from './queue';
import type { CodeSnapshot, CodeToken } from './types';

/** Preserves colored prefixes between coalesced full-block syntax snapshots. */
export class CodeSnapshots {
	private owner = {};
	private latest?: CodeSnapshot;
	private painted?: { text: string; language: CodeLanguage; paint: CodePaint };
	private timer?: ReturnType<typeof setTimeout>;
	private requested = false;

	constructor(
		private queue: HighlightQueue,
		private paint: (tokens: CodeToken[]) => CodePaint
	) {}

	update(snapshot: CodeSnapshot) {
		const previous = this.latest;
		const language = codeLanguage(snapshot.lang);
		this.latest = snapshot;
		if (this.painted) {
			if (language === this.painted.language && snapshot.text.startsWith(this.painted.text)) {
				this.painted.paint.refresh();
			} else {
				this.painted.paint.clear();
				this.painted = undefined;
			}
		}
		if (!language || language === 'plain' || !snapshot.text) {
			this.cancel();
			this.requested = false;
			return;
		}
		const appended =
			previous &&
			language === codeLanguage(previous.lang) &&
			snapshot.text.startsWith(previous.text);
		if (!this.requested || !snapshot.incremental || !appended) {
			this.cancel();
			this.submit();
		} else if (!this.timer) {
			const delay = snapshot.text.length > 262144 ? 1000 : snapshot.text.length > 65536 ? 250 : 100;
			this.timer = setTimeout(() => this.submit(), delay);
		}
	}

	destroy() {
		this.cancel();
		this.latest = undefined;
		this.painted?.paint.clear();
		this.painted = undefined;
	}

	private cancel() {
		clearTimeout(this.timer);
		this.timer = undefined;
		this.queue.cancel(this.owner);
	}

	private submit() {
		this.timer = undefined;
		const snapshot = this.latest;
		if (!snapshot) return;
		const language = codeLanguage(snapshot.lang);
		if (!language || language === 'plain') return;
		this.requested = true;
		this.queue.request(this.owner, snapshot.text, language, (tokens) => {
			if (
				!this.latest ||
				codeLanguage(this.latest.lang) !== language ||
				!this.latest.text.startsWith(snapshot.text) ||
				(this.painted && this.painted.text.length > snapshot.text.length)
			)
				return;
			const previous = this.painted;
			this.painted = { text: snapshot.text, language, paint: this.paint(tokens) };
			previous?.paint.clear();
		});
	}
}
