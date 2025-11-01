<script lang="ts">
	let { node, monochrome = false } = $props();
</script>

<span
	class="latex-span"
	class:monochrome
	style="font-family: 'Latin Modern Math', 'Times New Roman', serif;"
>
	<!-- Render inline latex, you may want to use a latex renderer here -->
	{node.text}
</span>
