<script>
	let { children } = $props();
</script>

<div class="flex h-screen flex-col justify-center">
	{@render children()}
</div>
