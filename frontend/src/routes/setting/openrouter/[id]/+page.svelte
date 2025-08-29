<script lang="ts">
	let { params } = $props();
	let id = $derived(Number(params.id));

	import { readModel } from '$lib/api/model';
	import { TiltBtn } from '$lib/components';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import { _ } from 'svelte-i18n';

	let config = $state('');

	let readModelPromise = $derived(readModel(id).then((x) => (config = x.raw)));

	let saveSetting = $_('setting.save_settings');
</script>

{#await readModelPromise}
	Loading
{:then _}
	{#key id}
		<ConfigEditor bind:value={config}>
			<TiltBtn
				class="rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
			>
				{saveSetting}
			</TiltBtn>
		</ConfigEditor>
	{/key}
{:catch}
	Error
{/await}
