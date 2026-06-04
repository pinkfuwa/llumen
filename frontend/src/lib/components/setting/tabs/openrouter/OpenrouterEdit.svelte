<script lang="ts">
	let { id, value = $bindable() }: { id: number; value: string } = $props();

	import { readModel, updateModel } from '$lib/api/model.svelte';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { m } from '$lib/paraglide/messages';

	let config = $state('');

	let readModelPromise = $derived(readModel(id).then((x) => (config = x.raw)));

	let saveSetting = $derived(m['setting.save_settings']());

	async function onSave() {
		const result = await updateModel({ id, config });
		if (result === 'success') value = 'openrouter';
	}
</script>

{#await readModelPromise}
	{m['common.loading']()}
{:then _}
	{#key id}
		<ConfigEditor bind:value={config}>
			<Button class="px-3 py-2" onclick={onSave}>
				{saveSetting}
			</Button>
		</ConfigEditor>
	{/key}
{:catch}
	{m['common.error']()}
{/await}
