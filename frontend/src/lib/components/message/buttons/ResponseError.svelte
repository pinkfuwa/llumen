<script lang="ts">
	import Warning from '$lib/components/setting/Warning.svelte';
	import { CloudAlert } from '@lucide/svelte';
	// import { _ } from 'svelte-i18n';
	let { content } = $props();
</script>

<Warning>
	<CloudAlert class="mr-2 inline-block h-7 w-7" />
	<!-- {$_('chat.error.no_output')} -->
	{content}
</Warning>
