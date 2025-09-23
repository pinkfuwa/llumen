<script lang="ts">
	const { children, thin = false } = $props();
</script>

<div
	class="flex items-center rounded-lg border-l-8 border-red-500 bg-component-bg font-semibold"
	class:p-2={thin}
	class:p-3={!thin}
>
	{@render children()}
</div>
