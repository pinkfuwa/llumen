<script lang="ts">
	import { checkConfig } from '../../api/model';
	import TiltBtn from '../buttons/TiltBtn.svelte';
	import Toml from '../codemirror/Toml.svelte';
	import { _ } from 'svelte-i18n';
	import Warning from './Warning.svelte';
	import { CircleCheck } from '@lucide/svelte';
	import { fade } from 'svelte/transition';

	let { value = $bindable(''), children } = $props();

	let { mutate } = checkConfig();

	let configChecked = $state(false);

	let configErrored = $state(false);
	let errorReason = $state('');
</script>

<Toml
	bind:value
	onchange={() => {
		configChecked = false;
		configErrored = false;
	}}
/>

<div class="mt-3 flex items-center justify-start space-x-2">
	<TiltBtn
		class="rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
		onclick={() =>
			mutate(
				{
					config: value
				},
				(x) => {
					if (x.reason == undefined) configChecked = true;
					else errorReason = x.reason;

					configErrored = !configChecked;
				}
			)}
	>
		{$_('setting.check_syntax')}
	</TiltBtn>
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
		<Warning message={errorReason} />
	</div>
{/if}
