<script lang="ts">
	import { preference } from '$lib/preference/index.svelte';
	import { onDestroy } from 'svelte';
	import { writable, toStore } from 'svelte/store';
	import { useModelIdsQueryEffect, getModelIds } from '$lib/api/model.svelte';
	import { _ } from 'svelte-i18n';

	const useCodeMirrorPromise = import('./index');
	useModelIdsQueryEffect();

	let {
		value = $bindable('# defaultConfig'),
		onchange = undefined as ((val: string) => void) | undefined
	} = $props();
	let div = $state<HTMLDivElement | null>(null);

	let valWritable = writable(value);
	$effect(() => valWritable.subscribe((newVal) => (value = newVal)));

	let callback: () => void | undefined;
	let loaded = $state(false);

	const darkTheme = $derived(preference.value.theme.dark);

	useCodeMirrorPromise.then((useCodeMirror) => {
		const modelIdsStore = toStore(() => getModelIds()?.ids ?? []);
		useCodeMirror.default({
			darkTheme: darkTheme,
			value: valWritable,
			element: toStore(() => div!),
			onDestroy: (x) => (callback = x),
			modelIds: modelIdsStore
		});
		loaded = true;
	});

	onDestroy(() => {
		if (callback != undefined) callback();
	});

	$effect(() => {
		if (!onchange) return;
		const unsubscriber = valWritable.subscribe((val) => onchange?.(val));
		return unsubscriber;
	});

	const themeStyle = $derived(
		darkTheme ? 'background-color:#24292e;color:#e1e4e8' : 'background-color:#fff;color:#24292e'
	);
</script>

<div class="border-radius-md h-full w-full rounded-md border border-border p-2" style={themeStyle}>
	{#if !loaded}
		<div class="h-full p-1.5 font-mono">{$_('common.loading_editor')}</div>
	{/if}
	<div bind:this={div} class="h-full shrink-0 space-y-2 [&>div]:h-full"></div>
</div>
