<script lang="ts">
	import { Markdown } from '$lib/components/markdown';
	import { useThrottle } from 'runed';
	import { untrack } from 'svelte';

	let content = $state('# markdown content');
	let renderContent = $state(untrack(() => content));

	const throttledUpdate = useThrottle(() => {
		renderContent = content;
	}, 230);
</script>

<div class="bg-chat relative grid h-screen w-screen grid-cols-2 gap-3 overflow-y-auto p-3">
	<title>Test Markdown</title>
	<textarea
		class="resize-none rounded-lg border border-outline bg-component-bg p-3 text-lg"
		bind:value={content}
		onkeyup={throttledUpdate}
	></textarea>
	<div class="overflow-y-auto rounded-lg border border-dashed border-outline p-3 pt-1">
		<div class="w-full">
			<Markdown source={renderContent} incremental />
		</div>
	</div>
</div>
