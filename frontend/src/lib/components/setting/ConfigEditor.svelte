<script lang="ts">
	import { checkConfig } from '../../api/model.svelte';
	import Toml from '../codemirror/Toml.svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import Warning from './Warning.svelte';
	import { CircleCheck } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import Button from '$lib/ui/Button.svelte';

	let { value = $bindable(''), children } = $props();

	let configChecked = $state(false);

	let configErrored = $state(false);
	let errorReason = $state('');

	async function onCheck() {
		configChecked = false;
		configErrored = false;
		const x = await checkConfig({ config: value });
		if (!x) return;
		if (x.reason == undefined) {
			configChecked = true;
			configErrored = false;
		} else {
			errorReason = x.reason;
			configErrored = true;
		}
	}
</script>

<Toml
	bind:value
	onchange={() => {
		configChecked = false;
		configErrored = false;
	}}
/>

<div class="mt-3 flex items-center justify-start space-x-2">
	<Button class="px-3 py-2" onclick={onCheck}>
		{m['setting.check_syntax'](lang)}
	</Button>
	{@render children()}
	{#if configChecked}
		<div
			class="ml-auto flex items-center justify-center pr-2"
			out:fade={{ duration: 300, delay: 100 }}
		>
			<CircleCheck class="mr-1 inline-block" />
			Checked!
		</div>
	{/if}
</div>

{#if configErrored}
	<div out:fade={{ duration: 300, delay: 100 }}>
		<Warning>
			{errorReason}
		</Warning>
	</div>
{/if}
