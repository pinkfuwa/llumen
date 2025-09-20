<script lang="ts">
	import { goto } from '$app/navigation';

	let { id, value = $bindable() }: { id: number; value: string } = $props();

	import { readModel, updateModel } from '$lib/api/model';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { _ } from 'svelte-i18n';

	let config = $state('');

	let readModelPromise = $derived(readModel(id).then((x) => (config = x.raw)));

	let saveSetting = $_('setting.save_settings');

	let { mutate } = updateModel();
</script>

{#await readModelPromise}
	Loading
{:then _}
	{#key id}
		<ConfigEditor bind:value={config}>
			<Button
				class="px-3 py-2"
				onclick={() => mutate({ id, config }, () => (value = 'openrouter'))}
			>
				{saveSetting}
			</Button>
		</ConfigEditor>
	{/key}
{:catch}
	Error
{/await}
