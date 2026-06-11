<script lang="ts">
	import { Markdown } from '$lib/components/markdown';

	let content = $state('# markdown content');
</script>

<div class="bg-chat relative grid h-screen w-screen grid-cols-2 gap-3 overflow-y-auto p-3">
	<title>Test Markdown</title>
	<textarea
		class="resize-none rounded-lg border border-border bg-card p-3 text-lg"
		bind:value={content}
	></textarea>
	<div class="overflow-y-auto rounded-lg border border-dashed border-border p-3 pt-1">
		<div class="w-full">
			<Markdown source={content} />
		</div>
	</div>
</div>
