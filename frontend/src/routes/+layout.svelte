<script>
	import Sidebar from '../components/Sidebar.svelte';
	import '../app.css';

	let { children } = $props();
</script>

<div class="flex h-screen flex-row">
	<Sidebar />
	<div>
		{@render children()}
	</div>
</div>
