/**
 * Simplified swipe gesture detection based on Hammer.js implementation
 * Supports left and right swipe gestures for sidebar open/close
 */

// Direction constants
const DIRECTION_NONE = 1;
const DIRECTION_LEFT = 2;
const DIRECTION_RIGHT = 4;
const DIRECTION_UP = 8;
const DIRECTION_DOWN = 16;
const DIRECTION_HORIZONTAL = DIRECTION_LEFT | DIRECTION_RIGHT;

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
function getDirection(deltaX: number, deltaY: number): number {
	if (deltaX === deltaY) {
		return DIRECTION_NONE;
	}

	if (Math.abs(deltaX) >= Math.abs(deltaY)) {
		return deltaX < 0 ? DIRECTION_LEFT : DIRECTION_RIGHT;
	}

	return deltaY < 0 ? DIRECTION_UP : DIRECTION_DOWN;
}

/**
 * Convert direction constant to string
 */
function directionToString(direction: number): SwipeDirection | null {
	if (direction === DIRECTION_LEFT) {
		return 'left';
	}
	if (direction === DIRECTION_RIGHT) {
		return 'right';
	}
	return null;
}

/**
 * Create a swipe gesture recognizer
 * Returns cleanup function to remove event listeners
 */
export function createSwipeGesture(element: HTMLElement, options: SwipeOptions): () => void {
	const threshold = options.threshold ?? 10;
	const velocityThreshold = options.velocity ?? 0.3;

	let touchState: TouchState | null = null;

	function handleTouchStart(event: TouchEvent): void {
		if (event.touches.length !== 1) {
			touchState = null;
			return;
		}

		const touch = event.touches[0];
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
			return;
		}

		const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);
		const velocity = distance / deltaTime;

		const direction = getDirection(deltaX, deltaY);

		// Check if it's a horizontal swipe with enough velocity and distance
		const isHorizontal = (direction & DIRECTION_HORIZONTAL) !== 0;
		const hasEnoughDistance = distance > threshold;
		const hasEnoughVelocity = velocity > velocityThreshold;

		if (isHorizontal && hasEnoughDistance && hasEnoughVelocity) {
			const directionString = directionToString(direction);
			if (directionString) {
				options.onSwipe(directionString);
			}
		}

		touchState = null;
	}

	function handleTouchCancel(): void {
		touchState = null;
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
