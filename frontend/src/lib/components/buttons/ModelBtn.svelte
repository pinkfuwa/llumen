<script lang="ts">
	import { ArrowUpDown } from '@lucide/svelte';
	let { model: value = $bindable('Default Model') } = $props();
</script>

<span class="inline-block items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
	>{value}
	<ArrowUpDown class="ml-2 inline-block h-[20px] w-[20px]" />
</span>
