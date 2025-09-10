<script lang="ts">
	import Code from '../../../markdown/Code.svelte';
	let { content }: { content: string } = $props();
</script>

<Code lang="json" text={content} copiable={false} />
