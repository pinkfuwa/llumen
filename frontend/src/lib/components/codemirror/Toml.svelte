<script lang="ts">
	import { isLightTheme } from '$lib/preference';
	import { onDestroy } from 'svelte';
	import { get, toStore, writable } from 'svelte/store';

	const useCodeMirrorPromise = import('./index');

	const defaultConfig = [
		'display_name="GPT-OSS 20B"',
		'# From https://openrouter.ai/models',
		'# don\'t put "online" suffix.',
		'openrouter_id="openai/gpt-oss-20b:free"',
		'',
		'[capability]',
		'# allow user to upload image, the model need to support it',
		'# set to false to disallow upload despite its support',
		'image = false',
		'audio = false',
		'# available option: Native, Text, Mistral, Disabled',
		'ocr = Native'
	].join('\n');
	let { value = $bindable(defaultConfig) } = $props();
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

	let themeStyle = $isLightTheme
		? 'background-color:#fff;color:#24292e'
		: 'background-color:#24292e;color:#e1e4e8';
</script>

<div
	class="border-radius-md w-full overflow-x-auto rounded-md border border-outline p-2"
	style={themeStyle}
>
	<div bind:this={div} class="h-full shrink-0 space-y-2"></div>
</div>
