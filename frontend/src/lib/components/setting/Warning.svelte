<script lang="ts">
	const { children } = $props();
</script>

<div class="mb-2 rounded-lg bg-red-700 hover:bg-red-500">
	<div class="ml-2 flex items-center bg-background p-3 font-semibold">
		{@render children()}
	</div>
</div>
