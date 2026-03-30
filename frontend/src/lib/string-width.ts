const segmenter = new Intl.Segmenter();

const zeroWidthClusterRegex =
	/^(?:\p{Default_Ignorable_Code_Point}|\p{Control}|\p{Format}|\p{Mark}|\p{Surrogate})+$/v;

const leadingNonPrintingRegex =
	/^[\p{Default_Ignorable_Code_Point}\p{Control}\p{Format}\p{Mark}\p{Surrogate}]+/v;

const rgiEmojiRegex = /^\p{RGI_Emoji}$/v;

const unqualifiedKeycapRegex = /^[\d#*]\u20E3$/;
const extendedPictographicRegex = /\p{Extended_Pictographic}/gu;

const asciiPrintableRegex = /^[\u0020-\u007E]*$/;

function isDoubleWidthNonRgiEmojiSequence(segment: string): boolean {
	if (segment.length > 50) {
		return false;
	}

	if (unqualifiedKeycapRegex.test(segment)) {
		return true;
	}

	if (segment.includes('\u200D')) {
		const pictographics = segment.match(extendedPictographicRegex);
		return pictographics !== null && pictographics.length >= 2;
	}

	return false;
}

function baseVisible(segment: string): string {
	return segment.replace(leadingNonPrintingRegex, '');
}

function isZeroWidthCluster(segment: string): boolean {
	return zeroWidthClusterRegex.test(segment);
}

function eastAsianWidth(codePoint: number, options: { ambiguousAsWide: boolean }): number {
	if (
		codePoint >= 0x1100 &&
		(codePoint <= 0x115f ||
			codePoint === 0x2329 ||
			codePoint === 0x232a ||
			(codePoint >= 0x2e80 && codePoint <= 0x303e) ||
			(codePoint >= 0x3041 && codePoint <= 0x33ff) ||
			(codePoint >= 0x3400 && codePoint <= 0x4dbf) ||
			(codePoint >= 0x4e00 && codePoint <= 0x9fff) ||
			(codePoint >= 0xa000 && codePoint <= 0xa4cf) ||
			(codePoint >= 0xac00 && codePoint <= 0xd7a3) ||
			(codePoint >= 0xf900 && codePoint <= 0xfaff) ||
			(codePoint >= 0xfe10 && codePoint <= 0xfe1f) ||
			(codePoint >= 0xfe30 && codePoint <= 0xfe6f) ||
			(codePoint >= 0xff00 && codePoint <= 0xff60) ||
			(codePoint >= 0xffe0 && codePoint <= 0xffe6) ||
			(codePoint >= 0x20000 && codePoint <= 0x2fffd) ||
			(codePoint >= 0x30000 && codePoint <= 0x3fffd))
	) {
		return 2;
	}

	if (
		(codePoint >= 0x3000 && codePoint <= 0x3003) ||
		(codePoint >= 0x3008 && codePoint <= 0x301b) ||
		(codePoint >= 0x301c && codePoint <= 0x301e) ||
		codePoint === 0x3020 ||
		codePoint === 0x3030 ||
		codePoint === 0x3036 ||
		codePoint === 0x3037 ||
		(codePoint >= 0xfe30 && codePoint <= 0xfe44) ||
		(codePoint >= 0xfe49 && codePoint <= 0xfe6f) ||
		(codePoint >= 0xff00 && codePoint <= 0xff0f) ||
		(codePoint >= 0xff1a && codePoint <= 0xff20) ||
		(codePoint >= 0xff3b && codePoint <= 0xff40) ||
		(codePoint >= 0xff5b && codePoint <= 0xff60) ||
		(codePoint >= 0xffe0 && codePoint <= 0xffe6) ||
		(codePoint >= 0xffe8 && codePoint <= 0xffee)
	) {
		return 2;
	}

	if (
		(codePoint >= 0x20a9 && codePoint <= 0x20c9) ||
		(codePoint >= 0x2100 && codePoint <= 0x214f) ||
		(codePoint >= 0x2150 && codePoint <= 0x218f) ||
		(codePoint >= 0x2190 && codePoint <= 0x2bff) ||
		(codePoint >= 0x2e80 && codePoint <= 0x2eff) ||
		(codePoint >= 0x2f00 && codePoint <= 0x2fdf) ||
		(codePoint >= 0x2ff0 && codePoint <= 0x2fff) ||
		(codePoint >= 0x3000 && codePoint <= 0x303f) ||
		(codePoint >= 0x3090 && codePoint <= 0x309f) ||
		(codePoint >= 0x30a0 && codePoint <= 0x30ff) ||
		(codePoint >= 0x3100 && codePoint <= 0x312f) ||
		(codePoint >= 0x3130 && codePoint <= 0x318f) ||
		(codePoint >= 0x3190 && codePoint <= 0x319f) ||
		(codePoint >= 0x31a0 && codePoint <= 0x31bf) ||
		(codePoint >= 0x31c0 && codePoint <= 0x31ef) ||
		(codePoint >= 0x31f0 && codePoint <= 0x31ff) ||
		(codePoint >= 0x3200 && codePoint <= 0x33ff) ||
		(codePoint >= 0xfe70 && codePoint <= 0xfeff) ||
		(codePoint >= 0xff50 && codePoint <= 0xff9f) ||
		(codePoint >= 0xfff0 && codePoint <= 0xffff)
	) {
		return options.ambiguousAsWide ? 2 : 1;
	}

	return 1;
}

function trailingHalfwidthWidth(
	segment: string,
	eastAsianWidthOptions: { ambiguousAsWide: boolean }
): number {
	let extra = 0;
	if (segment.length > 1) {
		for (const char of segment.slice(1)) {
			const code = char.codePointAt(0);
			if (code !== undefined && code >= 0xff00 && code <= 0xffef) {
				extra += eastAsianWidth(code, eastAsianWidthOptions);
			}
		}
	}
	return extra;
}

function stripAnsi(input: string): string {
	return input.replace(
		/[\x1b\x9b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]/g,
		''
	);
}

export interface StringWidthOptions {
	ambiguousIsNarrow?: boolean;
	countAnsiEscapeCodes?: boolean;
}

export function stringWidth(input: string, options: StringWidthOptions = {}): number {
	if (typeof input !== 'string' || input.length === 0) {
		return 0;
	}

	const { ambiguousIsNarrow = true, countAnsiEscapeCodes = false } = options;

	let string = input;

	if (!countAnsiEscapeCodes && (string.includes('\u001B') || string.includes('\u009B'))) {
		string = stripAnsi(string);
	}

	if (string.length === 0) {
		return 0;
	}

	if (asciiPrintableRegex.test(string)) {
		return string.length;
	}

	let width = 0;
	const eastAsianWidthOptions = { ambiguousAsWide: !ambiguousIsNarrow };

	for (const { segment } of segmenter.segment(string)) {
		if (isZeroWidthCluster(segment)) {
			continue;
		}

		if (rgiEmojiRegex.test(segment) || isDoubleWidthNonRgiEmojiSequence(segment)) {
			width += 2;
			continue;
		}

		const baseVisibleSegment = baseVisible(segment);
		const codePoint = baseVisibleSegment.codePointAt(0);
		if (codePoint !== undefined) {
			width += eastAsianWidth(codePoint, eastAsianWidthOptions);
		}

		width += trailingHalfwidthWidth(segment, eastAsianWidthOptions);
	}

	return width;
}

function segmentWidth(
	segment: string,
	eastAsianWidthOptions: { ambiguousAsWide: boolean }
): number {
	if (isZeroWidthCluster(segment)) {
		return 0;
	}

	if (rgiEmojiRegex.test(segment) || isDoubleWidthNonRgiEmojiSequence(segment)) {
		return 2;
	}

	const baseVisibleSegment = baseVisible(segment);
	const codePoint = baseVisibleSegment.codePointAt(0);
	const width = codePoint !== undefined ? eastAsianWidth(codePoint, eastAsianWidthOptions) : 0;
	return width + trailingHalfwidthWidth(segment, eastAsianWidthOptions);
}

export function stringWidthWithWrap(input: string, charsPerLine: number): number {
	if (!input || charsPerLine <= 0) {
		return 0;
	}

	const eastAsianWidthOptions = { ambiguousAsWide: false };

	const lines = input.split('\n');
	let totalRows = 0;

	for (const line of lines) {
		if (!line) {
			totalRows += 1;
			continue;
		}

		let lineWidth = 0;
		let lineRows = 1;

		for (const { segment } of segmenter.segment(line)) {
			const w = segmentWidth(segment, eastAsianWidthOptions);
			if (w === 0) continue;

			if (lineWidth + w > charsPerLine) {
				lineRows += 1;
				lineWidth = w;
			} else {
				lineWidth += w;
			}
		}

		totalRows += lineRows;
	}

	return totalRows;
}
