<script>
	import { storable } from '../../LocalStorage.svelte';
	let preference = storable({ theme: 'light' });
</script>

<div class="text-lg">
	<label for="theme">Theme: </label>
	<select id="theme" bind:value={$preference.theme}>
		<option value="light">Light</option>
		<option value="dark">Dark</option>
	</select>
</div>
