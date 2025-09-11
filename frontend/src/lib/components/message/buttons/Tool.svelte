<script lang="ts">
	import Code from '../../shiki/Code.svelte';
	let { content }: { content: string } = $props();
</script>

<Code lang="json" text={content} />
