<script>
	/**
	 * @typedef {Object} Props
	 * @property {string} [href]
	 * @property {any} [title]
	 * @property {string} [text]
	 */

	/** @type {Props} */
	let { href = '', title = undefined, text = '' } = $props();
</script>

<div class="flex justify-center">
	<img src={href} {title} alt={text} class="w-auto" style="max-height: 400px;" />
</div>
