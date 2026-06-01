<script lang="ts">
	const { children, thin = false } = $props();

	const warningStyle =
		'flex items-center rounded-lg border-l-8 border-destructive bg-card font-semibold';
</script>

<div class={warningStyle} class:p-2={thin} class:p-3={!thin}>
	{@render children()}
</div>
