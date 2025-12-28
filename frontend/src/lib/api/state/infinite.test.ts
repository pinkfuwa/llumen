import { describe, it, expect, beforeEach } from 'vitest';
import {
	insertInfiniteQueryData,
	updateInfiniteQueryDataById,
	removeInfiniteQueryData,
	getInfiniteQueryData,
	type PageState
} from './infinite.svelte';

interface TestItem {
	id: number;
	name: string;
}

describe('infinite query utilities', () => {
	let testPages: PageState<TestItem>[];

	beforeEach(() => {
		testPages = [
			{
				no: -1,
				startId: undefined,
				endId: 11,
				data: [{ id: 10, name: 'item10' }],
				target: null
			},
			{
				no: 0,
				startId: 10,
				endId: 6,
				data: [
					{ id: 9, name: 'item9' },
					{ id: 8, name: 'item8' },
					{ id: 7, name: 'item7' }
				],
				target: null
			},
			{
				no: 1,
				startId: 5,
				endId: undefined,
				data: [
					{ id: 5, name: 'item5' },
					{ id: 4, name: 'item4' },
					{ id: 3, name: 'item3' }
				],
				target: null
			}
		];
	});

	describe('insertInfiniteQueryData', () => {
		it('should insert data at the beginning of first page', () => {
			const newItem = { id: 11, name: 'item11' };
			const result = insertInfiniteQueryData(testPages, newItem);

			expect(result[0].data[0]).toEqual(newItem);
			expect(result[0].data[1]).toEqual({ id: 10, name: 'item10' });
		});

		it('should not insert if data already exists', () => {
			const existingItem = { id: 8, name: 'item8-duplicate' };
			const result = insertInfiniteQueryData(testPages, existingItem);

			expect(result).toEqual(testPages);
		});

		it('should return pages unchanged if pages array is empty', () => {
			const result = insertInfiniteQueryData([], { id: 1, name: 'item1' });

			expect(result).toEqual([]);
		});

		it('should not mutate original pages', () => {
			const originalFirstPageData = [...testPages[0].data];
			const newItem = { id: 11, name: 'item11' };

			insertInfiniteQueryData(testPages, newItem);

			expect(testPages[0].data).toEqual(originalFirstPageData);
		});
	});

	describe('updateInfiniteQueryDataById', () => {
		it('should update data by id in the correct page', () => {
			const result = updateInfiniteQueryDataById(testPages, 8, (item) => ({
				...item,
				name: 'updated-item8'
			}));

			const updatedItem = result[1].data.find((item) => item.id === 8);
			expect(updatedItem?.name).toBe('updated-item8');
		});

		it('should update multiple occurrences across pages', () => {
			const duplicatePages: PageState<TestItem>[] = [
				{
					no: 0,
					data: [
						{ id: 5, name: 'item5' },
						{ id: 4, name: 'item4' }
					],
					target: null
				},
				{
					no: 1,
					data: [
						{ id: 5, name: 'item5-duplicate' },
						{ id: 3, name: 'item3' }
					],
					target: null
				}
			];

			const result = updateInfiniteQueryDataById(duplicatePages, 5, (item) => ({
				...item,
				name: 'updated-item5'
			}));

			expect(result[0].data[0].name).toBe('updated-item5');
			expect(result[1].data[0].name).toBe('updated-item5');
		});

		it('should not modify pages without matching id', () => {
			const result = updateInfiniteQueryDataById(testPages, 999, (item) => ({
				...item,
				name: 'should-not-exist'
			}));

			expect(result).toEqual(testPages);
		});

		it('should not mutate original pages', () => {
			const originalData = JSON.parse(JSON.stringify(testPages));

			updateInfiniteQueryDataById(testPages, 8, (item) => ({
				...item,
				name: 'updated-item8'
			}));

			expect(testPages).toEqual(originalData);
		});
	});

	describe('removeInfiniteQueryData', () => {
		it('should remove data matching predicate from beginning', () => {
			const result = removeInfiniteQueryData(testPages, (item) => item.id >= 9);

			expect(result.length).toBe(2);
			expect(result[0].data).toEqual([
				{ id: 8, name: 'item8' },
				{ id: 7, name: 'item7' }
			]);
			expect(result[0].startId).toBeUndefined();
		});

		it('should remove entire pages if all data matches predicate', () => {
			const result = removeInfiniteQueryData(testPages, (item) => item.id >= 8);

			expect(result.length).toBe(2);
			expect(result[0].data).toEqual([{ id: 7, name: 'item7' }]);
			expect(result[1].data).toEqual([
				{ id: 5, name: 'item5' },
				{ id: 4, name: 'item4' },
				{ id: 3, name: 'item3' }
			]);
		});

		it('should return pages unchanged if no data matches predicate', () => {
			const result = removeInfiniteQueryData(testPages, (item) => item.id > 100);

			expect(result).toEqual(testPages);
		});

		it('should handle removal of all data', () => {
			const result = removeInfiniteQueryData(testPages, () => true);

			expect(result.length).toBe(0);
		});

		it('should not mutate original pages', () => {
			const originalData = JSON.parse(JSON.stringify(testPages));

			removeInfiniteQueryData(testPages, (item) => item.id >= 9);

			expect(testPages).toEqual(originalData);
		});

		it('should reset startId of first remaining page', () => {
			const result = removeInfiniteQueryData(testPages, (item) => item.id === 10);

			expect(result[0].startId).toBeUndefined();
		});
	});

	describe('getInfiniteQueryData', () => {
		it('should flatten all pages into single array', () => {
			const result = getInfiniteQueryData(testPages);

			expect(result).toEqual([
				{ id: 10, name: 'item10' },
				{ id: 9, name: 'item9' },
				{ id: 8, name: 'item8' },
				{ id: 7, name: 'item7' },
				{ id: 5, name: 'item5' },
				{ id: 4, name: 'item4' },
				{ id: 3, name: 'item3' }
			]);
		});

		it('should return empty array for empty pages', () => {
			const result = getInfiniteQueryData([]);

			expect(result).toEqual([]);
		});

		it('should handle pages with empty data arrays', () => {
			const pagesWithEmpty: PageState<TestItem>[] = [
				{
					no: 0,
					data: [{ id: 1, name: 'item1' }],
					target: null
				},
				{
					no: 1,
					data: [],
					target: null
				},
				{
					no: 2,
					data: [{ id: 2, name: 'item2' }],
					target: null
				}
			];

			const result = getInfiniteQueryData(pagesWithEmpty);

			expect(result).toEqual([
				{ id: 1, name: 'item1' },
				{ id: 2, name: 'item2' }
			]);
		});

		it('should maintain order of items', () => {
			const result = getInfiniteQueryData(testPages);

			// Verify descending order (as per pagination pattern)
			for (let i = 0; i < result.length - 1; i++) {
				expect(result[i].id).toBeGreaterThan(result[i + 1].id);
			}
		});
	});

	describe('integration scenarios', () => {
		it('should handle insert then update workflow', () => {
			const newItem = { id: 11, name: 'item11' };
			let pages = insertInfiniteQueryData(testPages, newItem);
			pages = updateInfiniteQueryDataById(pages, 11, (item) => ({
				...item,
				name: 'updated-item11'
			}));

			expect(pages[0].data[0]).toEqual({ id: 11, name: 'updated-item11' });
		});

		it('should handle insert then remove workflow', () => {
			const newItem = { id: 11, name: 'item11' };
			let pages = insertInfiniteQueryData(testPages, newItem);
			pages = removeInfiniteQueryData(pages, (item) => item.id === 11);

			expect(pages[0].data[0]).toEqual({ id: 10, name: 'item10' });
		});

		it('should handle update then remove workflow', () => {
			let pages = updateInfiniteQueryDataById(testPages, 10, (item) => ({
				...item,
				name: 'marked-for-deletion'
			}));
			pages = removeInfiniteQueryData(pages, (item) => item.name === 'marked-for-deletion');

			const allData = getInfiniteQueryData(pages);
			expect(allData.find((item) => item.id === 10)).toBeUndefined();
		});

		it('should maintain immutability through multiple operations', () => {
			const original = JSON.parse(JSON.stringify(testPages));

			let pages = insertInfiniteQueryData(testPages, { id: 11, name: 'item11' });
			pages = updateInfiniteQueryDataById(pages, 8, (item) => ({ ...item, name: 'updated' }));
			pages = removeInfiniteQueryData(pages, (item) => item.id === 11);

			expect(testPages).toEqual(original);
		});
	});

	describe('edge cases', () => {
		it('should handle single page with single item', () => {
			const singlePage: PageState<TestItem>[] = [
				{
					no: 0,
					data: [{ id: 1, name: 'item1' }],
					target: null
				}
			];

			const inserted = insertInfiniteQueryData(singlePage, { id: 2, name: 'item2' });
			expect(inserted[0].data.length).toBe(2);

			const updated = updateInfiniteQueryDataById(singlePage, 1, (item) => ({
				...item,
				name: 'updated'
			}));
			expect(updated[0].data[0].name).toBe('updated');

			const removed = removeInfiniteQueryData(singlePage, (item) => item.id === 1);
			expect(removed.length).toBe(0);
		});

		it('should handle pages with undefined start/end ids', () => {
			const pagesWithUndefined: PageState<TestItem>[] = [
				{
					no: 0,
					startId: undefined,
					endId: undefined,
					data: [{ id: 5, name: 'item5' }],
					target: null
				}
			];

			const result = insertInfiniteQueryData(pagesWithUndefined, { id: 6, name: 'item6' });
			expect(result[0].data[0]).toEqual({ id: 6, name: 'item6' });
		});

		it('should handle very large page numbers', () => {
			const largePages: PageState<TestItem>[] = [
				{
					no: 1000,
					data: [{ id: 1, name: 'item1' }],
					target: null
				},
				{
					no: 1001,
					data: [{ id: 2, name: 'item2' }],
					target: null
				}
			];

			const result = getInfiniteQueryData(largePages);
			expect(result.length).toBe(2);
		});

		it('should handle negative page numbers', () => {
			const negativePages: PageState<TestItem>[] = [
				{
					no: -2,
					data: [{ id: 3, name: 'item3' }],
					target: null
				},
				{
					no: -1,
					data: [{ id: 2, name: 'item2' }],
					target: null
				},
				{
					no: 0,
					data: [{ id: 1, name: 'item1' }],
					target: null
				}
			];

			const result = getInfiniteQueryData(negativePages);
			expect(result.length).toBe(3);
		});
	});
});
