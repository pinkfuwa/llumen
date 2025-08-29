<script lang="ts">
	import { isLightTheme } from '$lib/preference';
	import { onDestroy } from 'svelte';
	import { get, toStore, writable } from 'svelte/store';

	const useCodeMirrorPromise = import('./index');

	let {
		value = $bindable('# defaultConfig'),
		onchange = undefined as ((val: string) => void) | undefined
	} = $props();
	let div = $state<HTMLDivElement | null>(null);

	let valWritable = writable(value);
	$effect(() => valWritable.subscribe((newVal) => (value = newVal)));

	let callback: () => void | undefined;

	useCodeMirrorPromise.then((useCodeMirror) => {
		useCodeMirror.default({
			isLightTheme: get(isLightTheme),
			value: valWritable,
			element: toStore(() => div!),
			onDestroy: (x) => (callback = x)
		});
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

<div
	class="border-radius-md flex max-h-[480px] min-h-[200px] w-full flex-col overflow-auto rounded-md border border-outline p-2"
	style={themeStyle}
>
	<div bind:this={div} class="h-full shrink-0 space-y-2"></div>
</div>
