<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { useModelsQueryEffect, getModels } from '$lib/api/model.svelte.js';
	import { useUserQueryEffect, getCurrentUser } from '$lib/api/user.svelte.js';
	let { children, params } = $props();
	import { Sidebar } from '$lib/components';
	import OpenBtn from '$lib/components/sidebar/OpenBtn.svelte';
	import { createSwipeGesture } from '$lib/components/sidebar/gesture';
	import { onDestroy } from 'svelte';

	let addition = $derived(params.id != undefined);
	let open = $state(window.matchMedia('(width >= 48rem)').matches);
	let contentElement: HTMLElement;

	function uselessFn(i: any) {}
	$effect(() => {
		uselessFn(page.url);
		const large = window.matchMedia('(width >= 48rem)').matches;
		if (!large) open = false;
	});

	function onKeydown(e: KeyboardEvent) {
		if (e.ctrlKey && e.key === 's') {
			e.preventDefault();
			open = !open;
		} else if (e.ctrlKey && e.key === 'd') {
			e.preventDefault();
			open = false;
			goto('/chat/new');
		}
	}

	document.body.addEventListener('keydown', onKeydown);
	onDestroy(() => document.body.removeEventListener('keydown', onKeydown));

	useUserQueryEffect();
	useModelsQueryEffect();

	$effect(() => {
		if (!contentElement) {
			return;
		}

		const cleanup = createSwipeGesture(contentElement, {
			threshold: 50,
			velocity: 0.3,
			onSwipe: (direction) => {
				if (direction === 'right' && !open) {
					open = true;
				}
			}
		});

		return cleanup;
	});
</script>

<OpenBtn bind:open />

<div class="bg-chat relative flex h-screen w-screen flex-row">
	<div class="z-20 h-full shrink-0">
		<Sidebar {addition} bind:open />
	</div>

	<div bind:this={contentElement} class="absolute h-full w-full min-w-0 grow md:static md:w-auto">
		{@render children()}
	</div>
</div>
