<script>
	import Sidebar from '../../components/Sidebar.svelte';

	let { children } = $props();
</script>

<div class="flex h-screen flex-row">
	<Sidebar />
	<div class="grow">
		{@render children()}
	</div>
</div>
