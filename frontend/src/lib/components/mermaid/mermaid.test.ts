import { describe, test, expect } from 'vitest';
import { isMermaidLanguage, MERMAID_LANGUAGES } from './mermaid';

describe('Mermaid', () => {
	describe('isMermaidLanguage', () => {
		test('returns true for mermaid', () => {
			expect(isMermaidLanguage('mermaid')).toBe(true);
		});

		test('returns true for graph', () => {
			expect(isMermaidLanguage('graph')).toBe(true);
		});

		test('returns true for flowchart', () => {
			expect(isMermaidLanguage('flowchart')).toBe(true);
		});

		test('returns true for sequence', () => {
			expect(isMermaidLanguage('sequence')).toBe(true);
		});

		test('returns true for class', () => {
			expect(isMermaidLanguage('class')).toBe(true);
		});

		test('returns true for state', () => {
			expect(isMermaidLanguage('state')).toBe(true);
		});

		test('returns true for er', () => {
			expect(isMermaidLanguage('er')).toBe(true);
		});

		test('returns true for gantt', () => {
			expect(isMermaidLanguage('gantt')).toBe(true);
		});

		test('returns true for pie', () => {
			expect(isMermaidLanguage('pie')).toBe(true);
		});

		test('returns true for journey', () => {
			expect(isMermaidLanguage('journey')).toBe(true);
		});

		test('returns true for git', () => {
			expect(isMermaidLanguage('git')).toBe(true);
		});

		test('returns true for uppercase variants', () => {
			expect(isMermaidLanguage('GRAPH')).toBe(true);
			expect(isMermaidLanguage('Flowchart')).toBe(true);
			expect(isMermaidLanguage('MERMAID')).toBe(true);
		});

		test('returns false for undefined', () => {
			expect(isMermaidLanguage(undefined)).toBe(false);
		});

		test('returns false for empty string', () => {
			expect(isMermaidLanguage('')).toBe(false);
		});

		test('returns false for non-mermaid languages', () => {
			expect(isMermaidLanguage('javascript')).toBe(false);
			expect(isMermaidLanguage('python')).toBe(false);
			expect(isMermaidLanguage('html')).toBe(false);
			expect(isMermaidLanguage('rust')).toBe(false);
		});
	});

	describe('MERMAID_LANGUAGES', () => {
		test('contains expected languages', () => {
			expect(MERMAID_LANGUAGES.has('mermaid')).toBe(true);
			expect(MERMAID_LANGUAGES.has('graph')).toBe(true);
			expect(MERMAID_LANGUAGES.has('flowchart')).toBe(true);
			expect(MERMAID_LANGUAGES.has('sequence')).toBe(true);
			expect(MERMAID_LANGUAGES.has('class')).toBe(true);
			expect(MERMAID_LANGUAGES.has('state')).toBe(true);
			expect(MERMAID_LANGUAGES.has('er')).toBe(true);
			expect(MERMAID_LANGUAGES.has('gantt')).toBe(true);
			expect(MERMAID_LANGUAGES.has('pie')).toBe(true);
			expect(MERMAID_LANGUAGES.has('journey')).toBe(true);
			expect(MERMAID_LANGUAGES.has('git')).toBe(true);
		});
	});
});
