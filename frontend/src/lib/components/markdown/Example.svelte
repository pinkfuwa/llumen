<script lang="ts">
	import Root from './Root.svelte';

	let incrementalContent = $state('# Incremental Mode\n\n');
	let normalContent = $state('# Normal Mode\n\n');
	let isStreaming = $state(false);

	const demoText = `This is a **demonstration** of markdown parsing.

## Features

- *Italic text*
- **Bold text**
- \`inline code\`

### Math Support

The equation \\( E = mc^2 \\) is rendered inline.

Block equations work too:

$$
\\sum_{i=1}^n i = \\frac{n(n+1)}{2}
$$

### Code Example

\`\`\`javascript
function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}
\`\`\`

### Table

| Algorithm | Time | Space |
|-----------|------|-------|
| BFS | O(V+E) | O(V) |
| DFS | O(V+E) | O(V) |

> **Note**: Incremental mode is optimized for streaming content!
`;

	async function simulateStreaming() {
		if (isStreaming) return;

		isStreaming = true;
		incrementalContent = '# Incremental Mode\n\n';
		normalContent = '# Normal Mode\n\n';

		const chunks = demoText.split('');

		for (const char of chunks) {
			incrementalContent += char;
			normalContent += char;
			await new Promise((r) => setTimeout(r, 10));
		}

		isStreaming = false;
	}

	function reset() {
		incrementalContent = '# Incremental Mode\n\n';
		normalContent = '# Normal Mode\n\n';
		isStreaming = false;
	}
</script>

<div class="mx-auto max-w-6xl space-y-8 p-8">
	<div class="space-y-4">
		<h1 class="text-3xl font-bold">Markdown Parsing Modes</h1>
		<p class="text-gray-600">
			Compare incremental (optimized for streaming) vs normal (web worker) parsing modes.
		</p>
		<div class="flex gap-4">
			<button
				onclick={simulateStreaming}
				disabled={isStreaming}
				class="rounded bg-blue-500 px-4 py-2 text-white hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{isStreaming ? 'Streaming...' : 'Start Streaming Demo'}
			</button>
			<button
				onclick={reset}
				disabled={isStreaming}
				class="rounded bg-gray-500 px-4 py-2 text-white hover:bg-gray-600 disabled:cursor-not-allowed disabled:opacity-50"
			>
				Reset
			</button>
		</div>
	</div>

	<div class="grid grid-cols-1 gap-8 lg:grid-cols-2">
		<!-- Incremental Mode -->
		<div class="space-y-2">
			<div class="flex items-center gap-2">
				<h2 class="text-xl font-semibold">Incremental Mode</h2>
				<span class="rounded bg-green-50 px-2 py-1 text-sm text-green-600">Optimized</span>
			</div>
			<p class="text-sm text-gray-600">
				Parses in main thread with throttling. Reuses previous work.
			</p>
			<div class="min-h-[400px] overflow-auto rounded-lg border bg-white p-4 shadow-sm">
				<Root source={incrementalContent} incremental={true} />
			</div>
		</div>

		<!-- Normal Mode -->
		<div class="space-y-2">
			<div class="flex items-center gap-2">
				<h2 class="text-xl font-semibold">Normal Mode</h2>
				<span class="rounded bg-blue-50 px-2 py-1 text-sm text-blue-600">Default</span>
			</div>
			<p class="text-sm text-gray-600">Parses in web worker. Full reparse on every update.</p>
			<div class="min-h-[400px] overflow-auto rounded-lg border bg-white p-4 shadow-sm">
				<Root source={normalContent} incremental={false} />
			</div>
		</div>
	</div>

	<div class="rounded border-l-4 border-blue-500 bg-blue-50 p-4">
		<h3 class="mb-2 font-semibold">Performance Notes</h3>
		<ul class="space-y-1 text-sm text-gray-700">
			<li>
				<strong>Incremental mode</strong> is faster for streaming because it reuses parsing work from
				complete regions (paragraphs, code blocks, tables, etc.)
			</li>
			<li>
				<strong>Normal mode</strong> reparses the entire document but runs in a web worker to avoid blocking
				the UI
			</li>
			<li>Both modes use 100ms throttling to batch rapid updates during streaming</li>
			<li>
				For static content, either mode works well. For streaming, incremental is recommended.
			</li>
		</ul>
	</div>
</div>
