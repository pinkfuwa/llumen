const REGEXP_IMAGE_TYPE = /^image\/.+$/;

function isImageType(value: unknown): value is string {
	return typeof value === 'string' && REGEXP_IMAGE_TYPE.test(value);
}

function isPositiveNumber(value: unknown): value is number {
	return typeof value === 'number' && value > 0 && value < Infinity;
}

function toArray(value: ArrayLike<unknown>): unknown[] {
	return Array.from ? Array.from(value) : Array.prototype.slice.call(value);
}

function getStringFromCharCode(dataView: DataView, start: number, length: number): string {
	let str = '';
	for (let i = start; i < start + length; i++) {
		str += String.fromCharCode(dataView.getUint8(i));
	}
	return str;
}

function normalizeDecimalNumber(value: number, times = 100000000000): number {
	const regex = /\.\d*(?:0|9){12}\d*$/;
	return regex.test(String(value)) ? Math.round(value * times) / times : value;
}

function getAdjustedSizes(
	data: { aspectRatio: number; width?: number; height?: number },
	type = 'none'
): { width: number; height: number } {
	const { aspectRatio } = data;
	let { width, height } = data;

	if (isPositiveNumber(width) && isPositiveNumber(height)) {
		const adjustedWidth = height * aspectRatio;
		if ((type === 'contain' || type === 'none') && adjustedWidth > width) {
			height = width / aspectRatio;
		} else {
			width = height * aspectRatio;
		}
	} else if (isPositiveNumber(width)) {
		height = width / aspectRatio;
	} else if (isPositiveNumber(height)) {
		width = height * aspectRatio;
	}

	return { width: width ?? 0, height: height ?? 0 };
}

function resetAndGetOrientation(arrayBuffer: ArrayBuffer): number {
	const dataView = new DataView(arrayBuffer);
	let orientation = 1;

	try {
		let littleEndian: boolean;
		let app1Start = 0;

		if (dataView.getUint8(0) === 0xff && dataView.getUint8(1) === 0xd8) {
			const length = dataView.byteLength;
			let offset = 2;
			while (offset + 1 < length) {
				if (dataView.getUint8(offset) === 0xff && dataView.getUint8(offset + 1) === 0xe1) {
					app1Start = offset;
					break;
				}
				offset += 1;
			}
		}

		if (app1Start) {
			const exifIDCode = app1Start + 4;
			const tiffOffset = app1Start + 10;
			if (getStringFromCharCode(dataView, exifIDCode, 4) === 'Exif') {
				const endianness = dataView.getUint16(tiffOffset);
				littleEndian = endianness === 0x4949;
				if (littleEndian || endianness === 0x4d4d) {
					if (dataView.getUint16(tiffOffset + 2, littleEndian) === 0x002a) {
						const firstIFDOffset = dataView.getUint32(tiffOffset + 4, littleEndian);
						if (firstIFDOffset >= 0x00000008) {
							const ifdStart = tiffOffset + firstIFDOffset;
							const tagCount = dataView.getUint16(ifdStart, littleEndian);
							for (let i = 0; i < tagCount; i++) {
								const tagOffset = ifdStart + i * 12 + 2;
								if (dataView.getUint16(tagOffset, littleEndian) === 0x0112) {
									const valueOffset = tagOffset + 8;
									orientation = dataView.getUint16(valueOffset, littleEndian);
									dataView.setUint16(valueOffset, 1, littleEndian);
									break;
								}
							}
						}
					}
				}
			}
		}
	} catch {
		orientation = 1;
	}

	return orientation;
}

function parseOrientation(orientation: number): { rotate: number; scaleX: number; scaleY: number } {
	let rotate = 0;
	let scaleX = 1;
	let scaleY = 1;

	switch (orientation) {
		case 2:
			scaleX = -1;
			break;
		case 3:
			rotate = -180;
			break;
		case 4:
			scaleY = -1;
			break;
		case 5:
			rotate = 90;
			scaleY = -1;
			break;
		case 6:
			rotate = 90;
			break;
		case 7:
			rotate = 90;
			scaleX = -1;
			break;
		case 8:
			rotate = -90;
			break;
	}

	return { rotate, scaleX, scaleY };
}

interface CompressorOptions {
	quality?: number;
	maxWidth?: number;
	maxHeight?: number;
}

export async function compressImage(file: File, options: CompressorOptions = {}): Promise<File> {
	const { quality = 0.8 } = options;

	if (!isImageType(file.type)) {
		throw new Error('The first argument must be an image File or Blob object.');
	}

	const mimeType = file.type;
	const isJPEGImage = mimeType === 'image/jpeg';

	const orientation = isJPEGImage ? resetAndGetOrientation(await file.arrayBuffer()) : 1;
	const { rotate, scaleX, scaleY } = parseOrientation(orientation);

	return new Promise((resolve, reject) => {
		const image = new Image();

		image.onload = () => {
			const canvas = document.createElement('canvas');
			const context = canvas.getContext('2d');
			if (!context) {
				reject(new Error('Failed to get canvas context'));
				return;
			}

			const naturalWidth = image.naturalWidth;
			const naturalHeight = image.naturalHeight;

			const is90DegreesRotated = Math.abs(rotate) % 180 === 90;
			let effectiveMaxWidth = Math.max(options.maxWidth ?? Infinity, 0) || Infinity;
			let effectiveMaxHeight = Math.max(options.maxHeight ?? Infinity, 0) || Infinity;
			const minWidth = 0;
			const minHeight = 0;

			let width = naturalWidth;
			let height = naturalHeight;

			if (is90DegreesRotated) {
				[width, height] = [height, width];
				[effectiveMaxWidth, effectiveMaxHeight] = [effectiveMaxHeight, effectiveMaxWidth];
			}

			const aspectRatio = naturalWidth / naturalHeight;

			const adjustedMax = getAdjustedSizes(
				{ aspectRatio, width: effectiveMaxWidth, height: effectiveMaxHeight },
				'contain'
			);
			const adjustedMin = getAdjustedSizes(
				{ aspectRatio, width: minWidth, height: minHeight },
				'cover'
			);

			width = Math.min(Math.max(width, adjustedMin.width), adjustedMax.width);
			height = Math.min(Math.max(height, adjustedMin.height), adjustedMax.height);

			if (is90DegreesRotated) {
				[width, height] = [height, width];
			}

			width = Math.floor(normalizeDecimalNumber(width));
			height = Math.floor(normalizeDecimalNumber(height));

			canvas.width = width;
			canvas.height = height;

			context.fillStyle = isJPEGImage ? '#fff' : 'transparent';
			context.fillRect(0, 0, width, height);

			context.save();
			context.translate(width / 2, height / 2);
			context.rotate((rotate * Math.PI) / 180);
			context.scale(scaleX, scaleY);
			context.drawImage(image, -naturalWidth / 2, -naturalHeight / 2, naturalWidth, naturalHeight);
			context.restore();

			canvas.toBlob(
				(blob) => {
					if (!blob) {
						reject(new Error('Failed to compress image'));
						return;
					}

					try {
						const compressedFile = new File([blob], file.name, { type: blob.type });
						resolve(compressedFile);
					} catch {
						resolve(blob as unknown as File);
					}
				},
				mimeType,
				quality
			);
		};

		image.onabort = () => reject(new Error('Aborted to load the image.'));
		image.onerror = () => reject(new Error('Failed to load the image.'));

		image.src = URL.createObjectURL(file);
	});
}

export function isImageFile(file: File): boolean {
	return isImageType(file.type);
}
