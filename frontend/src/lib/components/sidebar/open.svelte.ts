import { page } from '$app/state';
import { createSwipeGesture } from '$lib/components/sidebar/gesture';

export const open = $state({ val: window.matchMedia('(width >= 48rem)').matches });

$effect.root(() => {
	$effect(() => {
		void page.url.pathname;
		const large = window.matchMedia('(width >= 48rem)').matches;
		if (!large) open.val = false;
	});

	$effect(() => {
		const cleanup = createSwipeGesture(document.body, {
			threshold: 50,
			velocity: 0.3,
			onSwipe: (direction) => {
				if (direction === 'right' && !open.val) {
					open.val = true;
				}
			}
		});

		return cleanup;
	});
});
