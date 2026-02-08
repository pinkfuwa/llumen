/**
 * Simplified swipe gesture detection based on Hammer.js implementation
 * Supports left and right swipe gestures for sidebar open/close
 */

export type SwipeDirection = 'left' | 'right';

interface TouchState {
	startX: number;
	startY: number;
	startTime: number;
	currentX: number;
	currentY: number;
	currentTime: number;
}

interface SwipeOptions {
	threshold?: number;
	velocity?: number;
	onSwipe: (direction: SwipeDirection) => void;
}

/**
 * Compute the direction based on deltaX and deltaY
 */
function getDirection(deltaX: number, deltaY: number): SwipeDirection | null {
	if (deltaX === deltaY) {
		return null;
	}

	if (Math.abs(deltaX) >= Math.abs(deltaY)) {
		return deltaX < 0 ? 'left' : 'right';
	}

	return null;
}

/**
 * Check if an element is horizontally scrollable
 */
function isHorizontallyScrollable(element: Element): boolean {
	return element.scrollWidth > element.clientWidth;
}

/**
 * Check if an element is scrolled to the end in the given direction
 */
function isScrolledToEnd(element: Element, direction: SwipeDirection): boolean {
	const scrollLeft = element.scrollLeft;
	const maxScroll = element.scrollWidth - element.clientWidth;

	if (direction === 'left') {
		// Swiping left means scrolling right, check if at right end
		return scrollLeft >= maxScroll - 1; // -1 for rounding errors
	} else {
		// Swiping right means scrolling left, check if at left end
		return scrollLeft <= 1; // Allow 1px tolerance
	}
}

/**
 * Find if any parent element (up to the container) blocks the gesture in the given direction
 */
function hasScrollableParentBlockingDirection(
	startElement: Element,
	container: HTMLElement,
	direction: SwipeDirection
): boolean {
	let current: Element | null = startElement;
	while (current && current !== container) {
		if (isHorizontallyScrollable(current)) {
			// If it's scrollable but not at the end in this direction, it blocks the gesture
			if (!isScrolledToEnd(current, direction)) {
				return true;
			}
		}
		current = current.parentElement;
	}
	return false;
}

/**
 * Create a swipe gesture recognizer
 * Returns cleanup function to remove event listeners
 */
export function createSwipeGesture(element: HTMLElement, options: SwipeOptions): () => void {
	const threshold = options.threshold ?? 10;
	const velocityThreshold = options.velocity ?? 0.3;

	let touchState: TouchState | null = null;
	let startTarget: Element | null = null;

	function handleTouchStart(event: TouchEvent): void {
		if (event.touches.length !== 1) {
			touchState = null;
			startTarget = null;
			return;
		}

		const touch = event.touches[0];
		if (typeof document.elementFromPoint === 'function') {
			startTarget = document.elementFromPoint(touch.clientX, touch.clientY);
		} else {
			startTarget = event.target instanceof Element ? event.target : null;
		}

		const now = Date.now();

		touchState = {
			startX: touch.clientX,
			startY: touch.clientY,
			startTime: now,
			currentX: touch.clientX,
			currentY: touch.clientY,
			currentTime: now
		};
	}

	function handleTouchMove(event: TouchEvent): void {
		if (!touchState || event.touches.length !== 1) {
			return;
		}

		const touch = event.touches[0];
		touchState.currentX = touch.clientX;
		touchState.currentY = touch.clientY;
		touchState.currentTime = Date.now();
	}

	function handleTouchEnd(event: TouchEvent): void {
		if (!touchState) {
			return;
		}

		const deltaX = touchState.currentX - touchState.startX;
		const deltaY = touchState.currentY - touchState.startY;
		const deltaTime = touchState.currentTime - touchState.startTime;

		// Prevent division by zero
		if (deltaTime === 0) {
			touchState = null;
			startTarget = null;
			return;
		}

		const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);
		const velocity = distance / deltaTime;

		const direction = getDirection(deltaX, deltaY);

		// Check if it's a horizontal swipe with enough velocity and distance
		const hasEnoughDistance = distance > threshold;
		const hasEnoughVelocity = velocity > velocityThreshold;

		if (direction && hasEnoughDistance && hasEnoughVelocity) {
			// Check if any parent element is scrollable and blocks this direction
			const isBlocked =
				startTarget && hasScrollableParentBlockingDirection(startTarget, element, direction);

			if (!isBlocked) {
				options.onSwipe(direction);
			}
		}

		touchState = null;
		startTarget = null;
	}

	function handleTouchCancel(): void {
		touchState = null;
		startTarget = null;
	}

	// Add event listeners
	element.addEventListener('touchstart', handleTouchStart, { passive: true });
	element.addEventListener('touchmove', handleTouchMove, { passive: true });
	element.addEventListener('touchend', handleTouchEnd, { passive: true });
	element.addEventListener('touchcancel', handleTouchCancel, { passive: true });

	// Return cleanup function
	return () => {
		element.removeEventListener('touchstart', handleTouchStart);
		element.removeEventListener('touchmove', handleTouchMove);
		element.removeEventListener('touchend', handleTouchEnd);
		element.removeEventListener('touchcancel', handleTouchCancel);
	};
}
