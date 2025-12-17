<script lang="ts">
	import { isLightTheme } from '$lib/preference';
	import { onDestroy } from 'svelte';
	import { derived, get, toStore, writable } from 'svelte/store';
	import { useModelIds } from '$lib/api/model';
	import { _ } from 'svelte-i18n';

	const useCodeMirrorPromise = import('./index');
	const modelIdsQuery = useModelIds();

	let {
		value = $bindable('# defaultConfig'),
		onchange = undefined as ((val: string) => void) | undefined
	} = $props();
	let div = $state<HTMLDivElement | null>(null);

	let valWritable = writable(value);
	$effect(() => valWritable.subscribe((newVal) => (value = newVal)));

	let callback: () => void | undefined;
	let loaded = $state(false);

	useCodeMirrorPromise.then((useCodeMirror) => {
		useCodeMirror.default({
			isLightTheme: get(isLightTheme),
			value: valWritable,
			element: toStore(() => div!),
			onDestroy: (x) => (callback = x),
			modelIds: derived(modelIdsQuery.data, (res) => res?.ids ?? [])
		});
		loaded = true;
	});

	onDestroy(() => {
		if (callback != undefined) callback();
	});

	if (onchange) {
		let unsubscriber = valWritable.subscribe(onchange);
		onDestroy(unsubscriber);
	}

	let themeStyle = $isLightTheme
		? 'background-color:#fff;color:#24292e'
		: 'background-color:#24292e;color:#e1e4e8';
</script>

<div class="border-radius-md h-full w-full rounded-md border border-outline p-2" style={themeStyle}>
	{#if !loaded}
		<div class="h-full p-1.5 font-mono">{$_('common.loading_editor')}</div>
	{/if}
	<div bind:this={div} class="h-full shrink-0 space-y-2 [&>div]:h-full"></div>
</div>
