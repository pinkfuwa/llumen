<script lang="ts">
	const { message }: { message: string } = $props();
</script>

<div class="mb-2 rounded-lg bg-red-700 hover:bg-red-500">
	<div class="ml-2 bg-background p-3 font-semibold">
		{message}
	</div>
</div>
