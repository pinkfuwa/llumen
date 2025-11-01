<script lang="ts">
	let { node, monochrome = false } = $props();
</script>

<div class:monochrome>
	<!-- Render LaTeX block, you may want to use a LaTeX renderer here -->
	<pre class="latex-block">{node.text}</pre>
</div>
