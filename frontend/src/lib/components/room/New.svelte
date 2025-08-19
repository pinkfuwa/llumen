<script>
	import { Plus } from '@lucide/svelte';
	import { slide } from 'svelte/transition';
</script>

<a
	href="/chat/new"
	class="mb-1 flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold hover:bg-hover"
	in:slide={{ duration: 120 }}
	out:slide={{ duration: 120 }}
>
	<Plus class="mr-2 h-5 w-5" />
	New
</a>
