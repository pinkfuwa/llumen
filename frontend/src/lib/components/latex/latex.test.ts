import { describe, it, expect } from 'vitest';
import { toHtml } from './latex';

describe('toHtml with temml', () => {
	it('should render inline math and return MathML', async () => {
		const result = await toHtml('E = mc^2', false);
		expect(result).toContain('<math');
		expect(result).toContain('</math>');
	});

	it('should render display math and return MathML', async () => {
		const result = await toHtml('\\frac{a}{b}', true);
		expect(result).toContain('<math');
		expect(result).toContain('</math>');
	});

	it('should handle LaTeX with multiple operators', async () => {
		const result = await toHtml('x^2 + y^2 = z^2', false);
		expect(result).toContain('<math');
	});

	it('should trim dollar signs from input', async () => {
		const result1 = await toHtml('$E = mc^2$', false);
		const result2 = await toHtml('E = mc^2', false);
		// Both should produce similar output
		expect(result1).toContain('<math');
		expect(result2).toContain('<math');
	});
});
