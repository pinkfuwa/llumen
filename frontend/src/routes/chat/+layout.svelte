<script lang="ts">
	let { children, params } = $props();
	import { Sidebar } from '$lib/components';

	let addition = $derived(params.id != undefined);
</script>

<div class="flex h-screen flex-row">
	<div class="shrink-0 overflow-hidden">
		<Sidebar {addition} currentRoom={Number(params.id)} />
	</div>
	<div class="h-screen w-full min-w-0 grow">
		{@render children()}
	</div>
</div>
