<script>
	import { Plus } from '@lucide/svelte';
	import { slide } from 'svelte/transition';
	import { _ } from 'svelte-i18n';
	import Button  from '$lib/ui/Button.svelte';
</script>

<Button
	href="/chat/new"
	class="flex w-full items-center justify-center rounded-md p-1.5 font-semibold"
>
	<Plus class="mr-2 h-5 w-5" />
	{$_('chat.new')}
</Button>
