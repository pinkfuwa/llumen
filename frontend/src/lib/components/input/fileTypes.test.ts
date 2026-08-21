import { describe, it, expect } from 'vitest';
import {
	getSupportedFileTypes,
	isMimeSupported,
	isFileTypeSupported,
	isTextFile,
	separateFiles,
	type CapabilityFileType
} from './fileTypes';

const none: CapabilityFileType = {
	image_input: false,
	audio_input: false,
	video_input: false,
	native_file_input: false,
	ocr_file_input: false
};

const allMedia: CapabilityFileType = {
	image_input: true,
	audio_input: true,
	video_input: true,
	native_file_input: true,
	ocr_file_input: true
};

function bytesOf(...values: number[]): ArrayBuffer {
	const bytes = new Uint8Array(values.length);
	bytes.set(values);
	return bytes.buffer;
}

function file(name: string, data: BlobPart, type = ''): File {
	return new File([data], name, { type, lastModified: 1 });
}

function utf8File(name: string, text: string, type = ''): File {
	return file(name, text, type);
}

describe('isMimeSupported', () => {
	it('does not substring-match extensions against MIME types', () => {
		expect(isMimeSupported('application/octet-stream', ['c'])).toBe(false);
		expect(isMimeSupported('application/zip', ['c', 'h', 'ts'])).toBe(false);
		expect(isMimeSupported('application/pdf', ['pdf'])).toBe(false);
	});

	it('matches exact MIMEs and type/* wildcards', () => {
		expect(isMimeSupported('application/pdf', ['application/pdf'])).toBe(true);
		expect(isMimeSupported('application/pdf; charset=binary', ['application/pdf'])).toBe(true);
		expect(isMimeSupported('audio/webm', ['audio/*'])).toBe(true);
		expect(isMimeSupported('image/png', ['image/*'])).toBe(true);
		expect(isMimeSupported('text/plain', ['image/*'])).toBe(false);
	});
});

describe('isFileTypeSupported', () => {
	const images = getSupportedFileTypes({ ...none, image_input: true });
	const pdf = getSupportedFileTypes({ ...none, native_file_input: true });
	const audio = getSupportedFileTypes({ ...none, audio_input: true });

	it('accepts images by MIME when image_input is on', () => {
		expect(isFileTypeSupported({ name: 'shot', type: 'image/png' }, images)).toBe(true);
		expect(isFileTypeSupported({ name: 'shot.png', type: '' }, images)).toBe(true);
		expect(isFileTypeSupported({ name: 'SHOT.PNG', type: '' }, images)).toBe(true);
		expect(isFileTypeSupported({ name: 'shot.png', type: '' }, getSupportedFileTypes(none))).toBe(
			false
		);
	});

	it('accepts audio/webm via audio/*', () => {
		expect(isFileTypeSupported({ name: 'clip.webm', type: 'audio/webm' }, audio)).toBe(true);
		expect(isFileTypeSupported({ name: 'clip.webm', type: 'audio/webm' }, images)).toBe(false);
	});

	it('accepts PDF only with native or OCR capability', () => {
		expect(isFileTypeSupported({ name: 'doc.pdf', type: 'application/pdf' }, pdf)).toBe(true);
		expect(
			isFileTypeSupported(
				{ name: 'doc.pdf', type: 'application/pdf' },
				getSupportedFileTypes({ ...none, ocr_file_input: true })
			)
		).toBe(true);
		expect(isFileTypeSupported({ name: 'doc.pdf', type: 'application/pdf' }, images)).toBe(false);
	});
});

describe('isTextFile', () => {
	it('accepts empty, UTF-8, UTF-8 BOM, and UTF-16 with BOM', async () => {
		expect(await isTextFile(file('empty.rs', bytesOf()))).toBe(true);
		expect(await isTextFile(utf8File('main.rs', 'fn main() {}\n'))).toBe(true);

		const hello = new TextEncoder().encode('hello');
		expect(await isTextFile(file('note.md', bytesOf(0xef, 0xbb, 0xbf, ...hello)))).toBe(true);

		expect(await isTextFile(file('wide.txt', bytesOf(0xff, 0xfe, 0x68, 0x00, 0x69, 0x00)))).toBe(
			true
		);
	});

	it('rejects NUL-containing and invalid UTF-8 bytes', async () => {
		expect(await isTextFile(file('foo.exe', bytesOf(0x4d, 0x5a, 0x00, 0x00)))).toBe(false);
		expect(await isTextFile(file('noise.bin', bytesOf(0x80, 0xff, 0x01)))).toBe(false);
	});
});

describe('separateFiles', () => {
	it('treats typical unknown binaries as unsupported even when MIME contains the letter c', async () => {
		const types = getSupportedFileTypes(allMedia);
		const exe = file(
			'foo.exe',
			bytesOf(0x4d, 0x5a, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00),
			'application/octet-stream'
		);
		const zip = file(
			'foo.zip',
			bytesOf(0x50, 0x4b, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00),
			'application/zip'
		);

		const { supported, unsupported } = await separateFiles([exe, zip], types);
		expect(supported).toEqual([]);
		expect(unsupported.map((f) => f.name)).toEqual(['foo.exe', 'foo.zip']);
	});

	it('accepts source and markdown by whole-file text, not extension', async () => {
		const types = getSupportedFileTypes(none);
		const rust = utf8File('main.rs', 'fn main() {}');
		const c = utf8File('foo.c', 'int main() { return 0; }');
		const md = utf8File('note.md', '# hello');
		const converted = utf8File('message.md', 'from composer', 'text/markdown');

		const { supported, unsupported } = await separateFiles([rust, c, md, converted], types);
		expect(unsupported).toEqual([]);
		expect(supported.map((f) => f.name)).toEqual(['main.rs', 'foo.c', 'note.md', 'message.md']);
	});

	it('gates images and PDFs on capability even when the name looks supported', async () => {
		const png = file(
			'shot.png',
			bytesOf(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a),
			'image/png'
		);
		const pdf = file('doc.pdf', bytesOf(0x25, 0x50, 0x44, 0x46, 0x2d, 0x00), 'application/pdf');

		const without = await separateFiles([png, pdf], getSupportedFileTypes(none));
		expect(without.supported).toEqual([]);
		expect(without.unsupported.map((f) => f.name)).toEqual(['shot.png', 'doc.pdf']);

		const withImage = await separateFiles(
			[png],
			getSupportedFileTypes({ ...none, image_input: true })
		);
		expect(withImage.supported.map((f) => f.name)).toEqual(['shot.png']);

		const withPdf = await separateFiles(
			[pdf],
			getSupportedFileTypes({ ...none, native_file_input: true })
		);
		expect(withPdf.supported.map((f) => f.name)).toEqual(['doc.pdf']);
	});
});
