<script lang="ts">
	import { copy } from '$lib/copy';
	import Root from '$lib/markdown/Root.svelte';
	import { CircleDollarSign, FolderSearch, CloudAlert } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import ErrorMessage from './ErrorMessage.svelte';
	import Warning from '../setting/Warning.svelte';
</script>

<div class="group w-full space-y-2 px-10 py-2 wrap-break-word lg:px-20 2xl:px-36">
	<div class="mb-2 border-b border-outline pb-2 select-none">
		<FolderSearch class="mr-2 inline-block" />
		{$_('chat.assistant.response')}
	</div>
	<div class="my-2 select-none">
		<Warning>
			<CloudAlert class="mr-2 inline-block h-7 w-7" />
			{$_('chat.error.no_output')}
		</Warning>
	</div>
</div>
