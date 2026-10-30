import { describe, expect, test } from 'vitest';
import { toHtml } from '../../components/latex/latex';

describe('LaTeX rendering', () => {
	test('does not inherit trusted rendering from Object.prototype', async () => {
		await toHtml('x', false);
		const originalTrust = Object.getOwnPropertyDescriptor(Object.prototype, 'trust');
		Object.defineProperty(Object.prototype, 'trust', {
			value: true,
			configurable: true,
			writable: true
		});

		try {
			const container = document.createElement('div');
			container.innerHTML = await toHtml('\\href{javascript:alert(1)}{click}', false);

			expect(container.querySelector('a')).toBeNull();
			expect(container.textContent).toContain('\\href');
		} finally {
			if (originalTrust) {
				Object.defineProperty(Object.prototype, 'trust', originalTrust);
			} else {
				Reflect.deleteProperty(Object.prototype, 'trust');
			}
		}
	});
});
