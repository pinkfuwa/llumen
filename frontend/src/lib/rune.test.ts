import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';
import { localState } from './rune.svelte';

let keyNumber = 0;
let key: string;
const listeners: EventListenerOrEventListenerObject[] = [];

beforeEach(() => {
	key = `local-state-test-${++keyNumber}`;
	localStorage.clear();
	const originalAddListener = window.addEventListener.bind(window);
	const addListener = vi.spyOn(window, 'addEventListener');
	addListener.mockImplementation((event, listener, options) => {
		if (event === 'storage') listeners.push(listener);
		originalAddListener(event, listener, options);
	});
});

afterEach(() => {
	flushSync();
	for (const listener of listeners.splice(0)) window.removeEventListener('storage', listener);
	vi.restoreAllMocks();
});

function createState() {
	return localState(key, {
		defaultValue: () => ({ locale: 'en-US' }),
		checker: (value) => typeof value?.locale === 'string'
	});
}

function changeStorage(eventKey: string, newValue: string | null) {
	window.dispatchEvent(new StorageEvent('storage', { key: eventKey, newValue }));
	flushSync();
}

describe('localState storage', () => {
	it('uses the default without overwriting storage on initialization', () => {
		const write = vi.spyOn(Storage.prototype, 'setItem');
		const state = createState();
		flushSync();
		expect(state.value).toEqual({ locale: 'en-US' });
		expect(write).not.toHaveBeenCalled();
	});

	it('restores a valid persisted value', () => {
		localStorage.setItem(key, JSON.stringify({ locale: 'zh-TW' }));
		expect(createState().value).toEqual({ locale: 'zh-TW' });
	});

	it.each(['{broken', 'null', '{"locale":42}'])(
		'falls back for invalid stored data (%s)',
		(raw) => {
			vi.spyOn(console, 'warn').mockImplementation(() => {});
			localStorage.setItem(key, raw);
			expect(createState().value).toEqual({ locale: 'en-US' });
		}
	);

	it('falls back when reading storage throws', () => {
		vi.spyOn(console, 'warn').mockImplementation(() => {});
		vi.spyOn(Storage.prototype, 'getItem').mockImplementationOnce(() => {
			throw new DOMException('Storage denied', 'SecurityError');
		});
		expect(createState().value).toEqual({ locale: 'en-US' });
	});

	it('persists nested changes once and ignores equivalent replacements', () => {
		const state = createState();
		flushSync();
		const write = vi.spyOn(Storage.prototype, 'setItem');
		state.value.locale = 'zh-TW';
		flushSync();
		expect(write).toHaveBeenCalledExactlyOnceWith(key, '{"locale":"zh-TW"}');
		state.value = { locale: 'zh-TW' };
		flushSync();
		expect(write).toHaveBeenCalledTimes(1);
	});

	it('keeps local changes when storage cannot be written', () => {
		const state = createState();
		flushSync();
		const warning = vi.spyOn(console, 'warn').mockImplementation(() => {});
		vi.spyOn(Storage.prototype, 'setItem').mockImplementationOnce(() => {
			throw new DOMException('Storage full', 'QuotaExceededError');
		});
		state.value.locale = 'zh-TW';
		expect(() => flushSync()).not.toThrow();
		expect(state.value.locale).toBe('zh-TW');
		expect(warning).toHaveBeenCalledOnce();
	});

	it('accepts changes from another tab without writing them back', () => {
		const state = createState();
		flushSync();
		const write = vi.spyOn(Storage.prototype, 'setItem');
		changeStorage(key, '{"locale":"zh-TW"}');
		expect(state.value.locale).toBe('zh-TW');
		expect(write).not.toHaveBeenCalled();
	});

	it.each(['{broken', 'null', '{"locale":42}', null])(
		'ignores invalid storage events (%s)',
		(raw) => {
			const state = createState();
			changeStorage(key, raw);
			expect(state.value.locale).toBe('en-US');
		}
	);

	it('ignores changes to another storage key', () => {
		const state = createState();
		changeStorage('another-key', '{"locale":"zh-TW"}');
		expect(state.value.locale).toBe('en-US');
	});
});

describe('localState remote synchronization', () => {
	function createSyncedState(remote: { locale: string } | null = { locale: 'zh-TW' }) {
		const syncer = {
			upload: vi.fn(async (_value: { locale: string }) => {}),
			download: vi.fn(async () => remote)
		};
		const state = localState(key, {
			defaultValue: () => ({ locale: 'en-US' }),
			checker: (value) => typeof value?.locale === 'string',
			syncer
		});
		flushSync();
		return { state, syncer };
	}

	it('uploads local edits and does not upload initial defaults', () => {
		const { state, syncer } = createSyncedState();
		expect(syncer.upload).not.toHaveBeenCalled();
		state.value.locale = 'fr-FR';
		flushSync();
		expect(syncer.upload).toHaveBeenCalledExactlyOnceWith({ locale: 'fr-FR' });
	});

	it('persists a downloaded value without uploading it again', async () => {
		const { state, syncer } = createSyncedState();
		await state.sync();
		flushSync();
		expect(state.value.locale).toBe('zh-TW');
		expect(localStorage.getItem(key)).toBe('{"locale":"zh-TW"}');
		expect(syncer.download).toHaveBeenCalledOnce();
		expect(syncer.upload).not.toHaveBeenCalled();
		state.value.locale = 'fr-FR';
		flushSync();
		expect(syncer.upload).toHaveBeenCalledExactlyOnceWith({ locale: 'fr-FR' });
	});

	it.each([null, { locale: 42 }])('ignores missing or invalid remote data (%s)', async (remote) => {
		const { state, syncer } = createSyncedState();
		syncer.download.mockResolvedValueOnce(remote as { locale: string } | null);
		await state.sync();
		flushSync();
		expect(state.value.locale).toBe('en-US');
		expect(localStorage.getItem(key)).toBeNull();
		expect(syncer.upload).not.toHaveBeenCalled();
	});

	it('does not upload changes received from another tab', () => {
		const { state, syncer } = createSyncedState();
		changeStorage(key, '{"locale":"zh-TW"}');
		expect(state.value.locale).toBe('zh-TW');
		expect(syncer.upload).not.toHaveBeenCalled();
	});

	it('preserves local data when downloading fails', async () => {
		const { state, syncer } = createSyncedState();
		syncer.download.mockRejectedValueOnce(new Error('offline'));
		await expect(state.sync()).rejects.toThrow('offline');
		expect(state.value.locale).toBe('en-US');
	});

	it('allows sync without a remote syncer', async () => {
		const state = createState();
		await expect(state.sync()).resolves.toBeUndefined();
		expect(state.value.locale).toBe('en-US');
	});
});
