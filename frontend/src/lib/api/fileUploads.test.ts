import { describe, expect, it, vi } from 'vitest';
import { createFileUploads, type UploadedFile } from './fileUploads';

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
	const prepareAndUpload =
		vi.fn<(file: File, signal: AbortSignal) => Promise<UploadedFile | null>>();
	prepareAndUpload.mockImplementation(async (file) => ({ name: file.name, id: 42 }));
	return {
		prepareAndUpload,
		uploads: createFileUploads(prepareAndUpload),
		file: new File(['hello'], 'note.txt')
	};
}

describe('file uploads', () => {
	it('returns no uploaded files for an empty selection', async () => {
		const { uploads, prepareAndUpload } = setup();
		await expect(uploads.waitForUploads([])).resolves.toEqual([]);
		expect(prepareAndUpload).not.toHaveBeenCalled();
	});

	it('starts uploads when setFiles is called', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		uploads.setFiles([file]);
		await Promise.resolve();
		expect(prepareAndUpload).toHaveBeenCalledWith(file, expect.any(AbortSignal));
		await expect(uploads.waitForUploads([file])).resolves.toEqual([{ name: file.name, id: 42 }]);
	});

	it('uses one upload per selected File object across repeated calls', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		uploads.setFiles([file]);
		uploads.setFiles([file]);
		await Promise.all([uploads.waitForUploads([file]), uploads.waitForUploads([file])]);
		await uploads.waitForUploads([file]);
		expect(prepareAndUpload).toHaveBeenCalledTimes(1);
	});

	it('uploads different files with the same name and size separately', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const other = new File(['world'], file.name);
		prepareAndUpload
			.mockResolvedValueOnce({ name: file.name, id: 1 })
			.mockResolvedValueOnce({ name: other.name, id: 2 });
		await expect(uploads.waitForUploads([file, other])).resolves.toEqual([
			{ name: file.name, id: 1 },
			{ name: other.name, id: 2 }
		]);
		expect(prepareAndUpload).toHaveBeenCalledTimes(2);
	});

	it('starts new uploads and copies the selection when waitForUploads is called', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const work = deferred<UploadedFile>();
		prepareAndUpload.mockReturnValue(work.promise);
		const files = [file];
		const result = uploads.waitForUploads(files);
		files.push(new File(['later'], 'later.txt'));
		work.resolve({ name: file.name, id: 42 });
		await expect(result).resolves.toEqual([{ name: file.name, id: 42 }]);
		expect(prepareAndUpload).toHaveBeenCalledTimes(1);
	});

	it('keeps selection order when uploads complete out of order', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const other = new File(['other'], 'other.txt');
		const first = deferred<UploadedFile>();
		const second = deferred<UploadedFile>();
		prepareAndUpload.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
		const result = uploads.waitForUploads([file, other]);
		await Promise.resolve();
		second.resolve({ name: other.name, id: 2 });
		await new Promise((resolve) => setTimeout(resolve, 0));
		first.resolve({ name: file.name, id: 1 });
		await expect(result).resolves.toEqual([
			{ name: file.name, id: 1 },
			{ name: other.name, id: 2 }
		]);
	});

	it('returns no files after removal while prepareAndUpload remains pending', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		prepareAndUpload.mockReturnValue(new Promise(() => {}));
		const result = uploads.waitForUploads([file]);
		await Promise.resolve();
		const signal = prepareAndUpload.mock.calls[0][1];
		uploads.setFiles([]);
		expect(signal.aborted).toBe(true);
		await expect(result).resolves.toEqual([]);
	});

	it('does not call prepareAndUpload for a file removed before the callback runs', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		uploads.setFiles([file]);
		uploads.setFiles([]);
		await uploads.waitForUploads([]);
		expect(prepareAndUpload).not.toHaveBeenCalled();
	});

	it('starts another upload when a removed File is selected again', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const oldWork = deferred<UploadedFile>();
		const newWork = deferred<UploadedFile>();
		prepareAndUpload.mockReturnValueOnce(oldWork.promise).mockReturnValueOnce(newWork.promise);
		const oldResult = uploads.waitForUploads([file]);
		await Promise.resolve();
		uploads.setFiles([]);
		const newResult = uploads.waitForUploads([file]);
		oldWork.resolve({ name: file.name, id: 1 });
		newWork.resolve({ name: file.name, id: 2 });
		await expect(oldResult).resolves.toEqual([]);
		await expect(newResult).resolves.toEqual([{ name: file.name, id: 2 }]);
		expect(prepareAndUpload).toHaveBeenCalledTimes(2);
	});

	it('omits a completed file removed while another upload is pending', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const other = new File(['other'], 'other.txt');
		const work = deferred<UploadedFile>();
		prepareAndUpload
			.mockResolvedValueOnce({ name: file.name, id: 1 })
			.mockReturnValueOnce(work.promise);
		const result = uploads.waitForUploads([file, other]);
		await new Promise((resolve) => setTimeout(resolve, 0));
		uploads.setFiles([other]);
		work.resolve({ name: other.name, id: 2 });
		await expect(result).resolves.toEqual([{ name: other.name, id: 2 }]);
	});

	it('omits null results without retrying retained failures', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const other = new File(['other'], 'other.txt');
		prepareAndUpload.mockResolvedValueOnce(null).mockResolvedValueOnce({ name: other.name, id: 2 });
		await expect(uploads.waitForUploads([file, other])).resolves.toEqual([
			{ name: other.name, id: 2 }
		]);
		await uploads.waitForUploads([file, other]);
		expect(prepareAndUpload).toHaveBeenCalledTimes(2);
	});

	it('handles an error before waitForUploads is called and rejects with that error', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const error = new TypeError('offline');
		prepareAndUpload.mockRejectedValue(error);
		uploads.setFiles([file]);
		await new Promise((resolve) => setTimeout(resolve, 0));
		await expect(uploads.waitForUploads([file])).rejects.toBe(error);
	});

	it('rejects waitForUploads when prepareAndUpload throws synchronously', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		const error = new Error('failed');
		prepareAndUpload.mockImplementation(() => {
			throw error;
		});
		await expect(uploads.waitForUploads([file])).rejects.toBe(error);
	});

	it('aborts all uploads, returns no pending files, and rejects calls after close', async () => {
		const { uploads, prepareAndUpload, file } = setup();
		prepareAndUpload.mockReturnValue(new Promise(() => {}));
		const result = uploads.waitForUploads([file]);
		await Promise.resolve();
		uploads.close();
		uploads.close();
		expect(prepareAndUpload.mock.calls[0][1].aborted).toBe(true);
		await expect(result).resolves.toEqual([]);
		expect(() => uploads.setFiles([file])).toThrow('closed');
		await expect(uploads.waitForUploads([file])).rejects.toThrow('closed');
		expect(prepareAndUpload).toHaveBeenCalledTimes(1);
	});
});
