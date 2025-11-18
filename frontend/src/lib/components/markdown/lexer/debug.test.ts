import { describe, it } from 'vitest';
import { parse, walkTree } from './parser';

describe('Debug LaTeX parsing', () => {
it('should debug simple case', async () => {
const source = '\\[x = 1\\]';
const tree = await parse(source);
const walked = await walkTree(tree, source);
console.log('Simple case:', JSON.stringify(walked, null, 2));
});

it('should debug with newline', async () => {
const source = 'Text:\n\\[x = 1\\]';
const tree = await parse(source);
const walked = await walkTree(tree, source);
console.log('With newline:', JSON.stringify(walked, null, 2));
});

it('should debug with blank line', async () => {
const source = 'Text:\n\n\\[x = 1\\]';
const tree = await parse(source);
const walked = await walkTree(tree, source);
console.log('With blank line:', JSON.stringify(walked, null, 2));
});

it('should debug problematic case', async () => {
const source = `Square both sides:

\\[
\\cos^2\\theta_x \\sin^2\\theta_y + \\sin^2\\theta_x
=
\\frac{1}{4} \\cos^2\\theta_y
\\tag{B}
\\]`;
const tree = await parse(source);
const walked = await walkTree(tree, source);
console.log('Problematic case:', JSON.stringify(walked, null, 2));
});
});
