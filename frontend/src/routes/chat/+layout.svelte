<script>
	import { goto } from '$app/navigation';
	import { token } from '$lib/store';

	let { children } = $props();

	$effect(() => {
		if ($token == undefined) goto('/login');
	});
</script>

{@render children()}
