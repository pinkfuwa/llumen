<script>
	import Sidebar from '../components/Sidebar.svelte';
	import '../app.css';
	import Setting from '../components/Setting/Setting.svelte';

	let { children } = $props();
</script>

<div class="flex h-screen flex-row">
	<Sidebar />
	<div class="grow">
		{@render children()}
	</div>
</div>
<Setting />
