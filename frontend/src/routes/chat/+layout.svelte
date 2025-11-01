<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { useModels } from '$lib/api/model.js';
	import { useUser } from '$lib/api/user.js';
	let { children, params } = $props();
	import { Sidebar } from '$lib/components';
	import CollapseBtn from '$lib/components/sidebar/CollapseBtn.svelte';
	import { onDestroy, setContext } from 'svelte';

	let addition = $derived(params.id != undefined);
	let open = $state(window.matchMedia('(width >= 48rem)').matches);

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

	const { data: user } = useUser();
	const { data: models } = useModels();
	setContext('user', user);
	setContext('models', models);
</script>

<CollapseBtn bind:open />

<div class="relative flex h-screen w-screen flex-row bg-chat-bg">
	<div class="z-20 h-full shrink-0">
		<Sidebar {addition} currentRoom={Number(params.id)} bind:open />
	</div>

	<div class="absolute h-full w-full min-w-0 grow md:static md:w-auto">
		{@render children()}
	</div>
</div>
