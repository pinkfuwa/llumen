<script lang="ts">
	import { preference } from '$lib/preference/index.svelte';
	import { onDestroy } from 'svelte';
	import { writable, toStore } from 'svelte/store';
	import { modelIds } from '$lib/api/model.svelte';
	import { t } from 'svelte-intl-precompile';

	const useCodeMirrorPromise = import('./index');

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
	const themeName = $derived(preference.value.theme.name);

	useCodeMirrorPromise.then((useCodeMirror) => {
		const modelIdsStore = toStore(() => modelIds.val ?? []);
		useCodeMirror.default({
			darkTheme: darkTheme,
			themeName: themeName,
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
		darkTheme
			? themeName === 'vitesse'
				? 'background-color:#1e1e1e;color:#d4cfbf'
				: 'background-color:#24292e;color:#e1e4e8'
			: themeName === 'vitesse'
				? 'background-color:#fbfbfb;color:#393a34'
				: 'background-color:#fff;color:#24292e'
	);
</script>

<div class="border-radius-md h-full w-full rounded-md border border-border p-2" style={themeStyle}>
	{#if !loaded}
		<div class="h-full p-1.5 font-mono">{$t('common.loading_editor')}</div>
	{/if}
	<div bind:this={div} class="h-full shrink-0 space-y-2 [&>div]:h-full"></div>
</div>
