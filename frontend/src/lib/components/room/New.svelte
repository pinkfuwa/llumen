<script>
	import { Plus } from '@lucide/svelte';
	import { slide } from 'svelte/transition';
	import { _ } from 'svelte-i18n';
</script>

<a
	href="/chat/new"
	class="flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold select-none hover:bg-hover"
	in:slide={{ duration: 150 }}
	out:slide={{ duration: 120 }}
>
	<Plus class="mr-2 h-5 w-5" />
	{$_('chat.file')}
</a>
