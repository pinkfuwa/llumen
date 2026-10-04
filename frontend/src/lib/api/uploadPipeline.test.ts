import { describe, expect, it, vi } from 'vitest';
import { createUploadQueue, type UploadedFile } from './uploadPipeline';

function deferred<T>() {
	let resolve!: (value: T) => void;
	let reject!: (reason: unknown) => void;
	const promise = new Promise<T>((resolvePromise, rejectPromise) => {
		resolve = resolvePromise;
		reject = rejectPromise;
	});
	return { promise, resolve, reject };
}

function setup() {
	const process = vi.fn<(file: File, signal: AbortSignal) => Promise<UploadedFile | null>>();
	process.mockImplementation(async (file) => ({ name: file.name, id: 42 }));
	return { process, queue: createUploadQueue(process), file: new File(['hello'], 'note.txt') };
}

describe('upload queue', () => {
	it('returns an empty selection without starting work', async () => {
		const { queue, process } = setup();
		await expect(queue.ready([])).resolves.toEqual([]);
		expect(process).not.toHaveBeenCalled();
	});

	it('starts uploads on update before ready is called', async () => {
		const { queue, process, file } = setup();
		queue.update([file]);
		await Promise.resolve();
		expect(process).toHaveBeenCalledWith(file, expect.any(AbortSignal));
		await expect(queue.ready([file])).resolves.toEqual([{ name: file.name, id: 42 }]);
	});

	it('shares retained jobs across updates and concurrent reads', async () => {
		const { queue, process, file } = setup();
		queue.update([file]);
		queue.update([file]);
		await Promise.all([queue.ready([file]), queue.ready([file])]);
		await queue.ready([file]);
		expect(process).toHaveBeenCalledTimes(1);
	});

	it('uploads different files with the same name and size separately', async () => {
		const { queue, process, file } = setup();
		const other = new File(['world'], file.name);
		process
			.mockResolvedValueOnce({ name: file.name, id: 1 })
			.mockResolvedValueOnce({ name: other.name, id: 2 });
		await expect(queue.ready([file, other])).resolves.toEqual([
			{ name: file.name, id: 1 },
			{ name: other.name, id: 2 }
		]);
		expect(process).toHaveBeenCalledTimes(2);
	});

	it('starts missing jobs and captures selection at ready call time', async () => {
		const { queue, process, file } = setup();
		const work = deferred<UploadedFile>();
		process.mockReturnValue(work.promise);
		const files = [file];
		const result = queue.ready(files);
		files.push(new File(['later'], 'later.txt'));
		work.resolve({ name: file.name, id: 42 });
		await expect(result).resolves.toEqual([{ name: file.name, id: 42 }]);
		expect(process).toHaveBeenCalledTimes(1);
	});

	it('keeps selection order when uploads complete out of order', async () => {
		const { queue, process, file } = setup();
		const other = new File(['other'], 'other.txt');
		const first = deferred<UploadedFile>();
		const second = deferred<UploadedFile>();
		process.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
		const result = queue.ready([file, other]);
		second.resolve({ name: other.name, id: 2 });
		first.resolve({ name: file.name, id: 1 });
		await expect(result).resolves.toEqual([
			{ name: file.name, id: 1 },
			{ name: other.name, id: 2 }
		]);
	});

	it('settles removed jobs even when processing never completes', async () => {
		const { queue, process, file } = setup();
		process.mockReturnValue(new Promise(() => {}));
		const result = queue.ready([file]);
		await Promise.resolve();
		const signal = process.mock.calls[0][1];
		queue.update([]);
		expect(signal.aborted).toBe(true);
		await expect(result).resolves.toEqual([]);
	});

	it('does not start work removed before processing begins', async () => {
		const { queue, process, file } = setup();
		queue.update([file]);
		queue.update([]);
		await queue.ready([]);
		expect(process).not.toHaveBeenCalled();
	});

	it('starts a fresh job on re-add and ignores the cancelled completion', async () => {
		const { queue, process, file } = setup();
		const oldWork = deferred<UploadedFile>();
		const newWork = deferred<UploadedFile>();
		process.mockReturnValueOnce(oldWork.promise).mockReturnValueOnce(newWork.promise);
		const oldResult = queue.ready([file]);
		await Promise.resolve();
		queue.update([]);
		const newResult = queue.ready([file]);
		oldWork.resolve({ name: file.name, id: 1 });
		newWork.resolve({ name: file.name, id: 2 });
		await expect(oldResult).resolves.toEqual([]);
		await expect(newResult).resolves.toEqual([{ name: file.name, id: 2 }]);
		expect(process).toHaveBeenCalledTimes(2);
	});

	it('omits a completed file removed while another upload is pending', async () => {
		const { queue, process, file } = setup();
		const other = new File(['other'], 'other.txt');
		const work = deferred<UploadedFile>();
		process.mockResolvedValueOnce({ name: file.name, id: 1 }).mockReturnValueOnce(work.promise);
		const result = queue.ready([file, other]);
		await Promise.resolve();
		queue.update([other]);
		work.resolve({ name: other.name, id: 2 });
		await expect(result).resolves.toEqual([{ name: other.name, id: 2 }]);
	});

	it('omits null results without retrying retained failures', async () => {
		const { queue, process, file } = setup();
		const other = new File(['other'], 'other.txt');
		process.mockResolvedValueOnce(null).mockResolvedValueOnce({ name: other.name, id: 2 });
		await expect(queue.ready([file, other])).resolves.toEqual([{ name: other.name, id: 2 }]);
		await queue.ready([file, other]);
		expect(process).toHaveBeenCalledTimes(2);
	});

	it('observes background rejections and still rejects ready with the original error', async () => {
		const { queue, process, file } = setup();
		const error = new TypeError('offline');
		process.mockRejectedValue(error);
		queue.update([file]);
		await new Promise((resolve) => setTimeout(resolve, 0));
		await expect(queue.ready([file])).rejects.toBe(error);
	});

	it('rejects ready when processing throws synchronously', async () => {
		const { queue, process, file } = setup();
		const error = new Error('failed');
		process.mockImplementation(() => {
			throw error;
		});
		await expect(queue.ready([file])).rejects.toBe(error);
	});

	it('disposes all jobs, settles waiters, and prevents new work', async () => {
		const { queue, process, file } = setup();
		process.mockReturnValue(new Promise(() => {}));
		const result = queue.ready([file]);
		await Promise.resolve();
		queue.dispose();
		queue.dispose();
		expect(process.mock.calls[0][1].aborted).toBe(true);
		await expect(result).resolves.toEqual([]);
		expect(() => queue.update([file])).toThrow('disposed');
		await expect(queue.ready([file])).rejects.toThrow('disposed');
		expect(process).toHaveBeenCalledTimes(1);
	});
});
