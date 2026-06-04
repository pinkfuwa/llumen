import { prepareWithSegments, measureNaturalWidth, measureLineStats } from '@chenglou/pretext';

export function stringWidth(input: string, font: string): number {
	if (!input || !font) return 0;
	const prepared = prepareWithSegments(input, font);
	return measureNaturalWidth(prepared);
}

export function stringWidthWithWrap(input: string, font: string, maxWidthPx: number): number {
	if (!input || !font || maxWidthPx <= 0) return 0;
	const prepared = prepareWithSegments(input, font);
	const stats = measureLineStats(prepared, maxWidthPx);
	return stats.lineCount;
}
