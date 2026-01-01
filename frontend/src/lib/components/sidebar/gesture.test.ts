import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createSwipeGesture, type SwipeDirection } from './gesture';

describe('gesture detection', () => {
	let element: HTMLElement;
	let onSwipe: ReturnType<typeof vi.fn>;
	let cleanup: (() => void) | undefined;
	let mockTime: number;

	beforeEach(() => {
		element = document.createElement('div');
		document.body.appendChild(element);
		onSwipe = vi.fn();
		cleanup = undefined;
		mockTime = 1000;
		vi.spyOn(Date, 'now').mockImplementation(() => mockTime);
	});

	afterEach(() => {
		if (cleanup) {
			cleanup();
		}
		if (element.parentNode) {
			document.body.removeChild(element);
		}
		vi.restoreAllMocks();
	});

	function simulateSwipe(
		startX: number,
		startY: number,
		endX: number,
		endY: number,
		delay: number = 100
	): void {
		const touchStart = new TouchEvent('touchstart', {
			touches: [
				{
					clientX: startX,
					clientY: startY,
					identifier: 0,
					target: element
				} as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchStart);

		// Simulate some time passing and movement
		setTimeout(() => {
			const touchMove = new TouchEvent('touchmove', {
				touches: [
					{
						clientX: endX,
						clientY: endY,
						identifier: 0,
						target: element
					} as unknown as Touch
				],
				bubbles: true
			});
			element.dispatchEvent(touchMove);
		}, delay / 2);

		setTimeout(() => {
			const touchEnd = new TouchEvent('touchend', {
				changedTouches: [
					{
						clientX: endX,
						clientY: endY,
						identifier: 0,
						target: element
					} as unknown as Touch
				],
				bubbles: true
			});
			element.dispatchEvent(touchEnd);
		}, delay);
	}

	it('should detect left swipe with fast gesture', () => {
		cleanup = createSwipeGesture(element, {
			onSwipe: onSwipe as (direction: SwipeDirection) => void
		});

		// Fast left swipe: 200px -> 50px in short time
		const touchStart = new TouchEvent('touchstart', {
			touches: [{ clientX: 200, clientY: 100, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchStart);

		// Advance time by 50ms
		mockTime += 50;

		const touchMove = new TouchEvent('touchmove', {
			touches: [{ clientX: 50, clientY: 100, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchMove);

		// Advance time by another 50ms (100ms total)
		mockTime += 50;

		const touchEnd = new TouchEvent('touchend', {
			changedTouches: [
				{ clientX: 50, clientY: 100, identifier: 0, target: element } as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchEnd);

		expect(onSwipe).toHaveBeenCalledWith('left');
	});

	it('should detect right swipe with fast gesture', () => {
		cleanup = createSwipeGesture(element, {
			onSwipe: onSwipe as (direction: SwipeDirection) => void
		});

		const touchStart = new TouchEvent('touchstart', {
			touches: [{ clientX: 50, clientY: 100, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchStart);

		// Advance time by 50ms
		mockTime += 50;

		const touchMove = new TouchEvent('touchmove', {
			touches: [{ clientX: 200, clientY: 100, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchMove);

		// Advance time by another 50ms (100ms total)
		mockTime += 50;

		const touchEnd = new TouchEvent('touchend', {
			changedTouches: [
				{ clientX: 200, clientY: 100, identifier: 0, target: element } as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchEnd);

		expect(onSwipe).toHaveBeenCalledWith('right');
	});

	it('should not trigger swipe with insufficient distance', () => {
		cleanup = createSwipeGesture(element, {
			onSwipe: onSwipe as (direction: SwipeDirection) => void,
			threshold: 100
		});

		const touchStart = new TouchEvent('touchstart', {
			touches: [{ clientX: 100, clientY: 100, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchStart);

		mockTime += 50;

		const touchMove = new TouchEvent('touchmove', {
			touches: [{ clientX: 130, clientY: 100, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchMove);

		mockTime += 50;

		const touchEnd = new TouchEvent('touchend', {
			changedTouches: [
				{ clientX: 130, clientY: 100, identifier: 0, target: element } as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchEnd);

		expect(onSwipe).not.toHaveBeenCalled();
	});

	it('should not trigger swipe for primarily vertical movement', () => {
		cleanup = createSwipeGesture(element, {
			onSwipe: onSwipe as (direction: SwipeDirection) => void
		});

		const touchStart = new TouchEvent('touchstart', {
			touches: [{ clientX: 100, clientY: 50, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchStart);

		mockTime += 50;

		const touchMove = new TouchEvent('touchmove', {
			touches: [{ clientX: 100, clientY: 200, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchMove);

		mockTime += 50;

		const touchEnd = new TouchEvent('touchend', {
			changedTouches: [
				{ clientX: 100, clientY: 200, identifier: 0, target: element } as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchEnd);

		expect(onSwipe).not.toHaveBeenCalled();
	});

	it('should ignore multi-touch gestures', () => {
		cleanup = createSwipeGesture(element, {
			onSwipe: onSwipe as (direction: SwipeDirection) => void
		});

		const touchStart = new TouchEvent('touchstart', {
			touches: [
				{ clientX: 100, clientY: 100, identifier: 0, target: element } as unknown as Touch,
				{ clientX: 150, clientY: 100, identifier: 1, target: element } as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchStart);

		const touchEnd = new TouchEvent('touchend', {
			changedTouches: [
				{ clientX: 50, clientY: 100, identifier: 0, target: element } as unknown as Touch,
				{ clientX: 100, clientY: 100, identifier: 1, target: element } as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchEnd);

		expect(onSwipe).not.toHaveBeenCalled();
	});

	it('should cleanup event listeners on cleanup call', () => {
		const addEventListenerSpy = vi.spyOn(element, 'addEventListener');
		const removeEventListenerSpy = vi.spyOn(element, 'removeEventListener');

		cleanup = createSwipeGesture(element, {
			onSwipe: onSwipe as (direction: SwipeDirection) => void
		});

		expect(addEventListenerSpy).toHaveBeenCalledWith('touchstart', expect.any(Function), {
			passive: true
		});
		expect(addEventListenerSpy).toHaveBeenCalledWith('touchmove', expect.any(Function), {
			passive: true
		});
		expect(addEventListenerSpy).toHaveBeenCalledWith('touchend', expect.any(Function), {
			passive: true
		});
		expect(addEventListenerSpy).toHaveBeenCalledWith('touchcancel', expect.any(Function), {
			passive: true
		});

		cleanup();

		expect(removeEventListenerSpy).toHaveBeenCalledWith('touchstart', expect.any(Function));
		expect(removeEventListenerSpy).toHaveBeenCalledWith('touchmove', expect.any(Function));
		expect(removeEventListenerSpy).toHaveBeenCalledWith('touchend', expect.any(Function));
		expect(removeEventListenerSpy).toHaveBeenCalledWith('touchcancel', expect.any(Function));
	});

	it('should reset state on touchcancel', () => {
		cleanup = createSwipeGesture(element, {
			onSwipe: onSwipe as (direction: SwipeDirection) => void
		});

		const touchStart = new TouchEvent('touchstart', {
			touches: [{ clientX: 200, clientY: 100, identifier: 0, target: element } as unknown as Touch],
			bubbles: true
		});
		element.dispatchEvent(touchStart);

		mockTime += 50;

		const touchCancel = new TouchEvent('touchcancel', {
			bubbles: true
		});
		element.dispatchEvent(touchCancel);

		mockTime += 50;

		// Now end should not trigger swipe
		const touchEnd = new TouchEvent('touchend', {
			changedTouches: [
				{ clientX: 50, clientY: 100, identifier: 0, target: element } as unknown as Touch
			],
			bubbles: true
		});
		element.dispatchEvent(touchEnd);

		expect(onSwipe).not.toHaveBeenCalled();
	});
});
