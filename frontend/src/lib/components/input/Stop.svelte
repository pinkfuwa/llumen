<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import { Square } from '@lucide/svelte';
	import { abortStream } from './state.svelte';
</script>

<Button class="-mr-2 p-2" onclick={abortStream} borderless>
	<Square class="inline-block" />
</Button>
