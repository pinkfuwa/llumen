<script lang="ts">
	import { Tooltip } from 'bits-ui';

	let { children, text } = $props();
</script>

<Tooltip.Provider>
	<Tooltip.Root delayDuration={200}>
		<Tooltip.Trigger>
			{@render children()}
		</Tooltip.Trigger>
		<Tooltip.Content sideOffset={8}>
			<div class="rounded-md bg-popup-bg p-2">
				{text}
			</div>
		</Tooltip.Content>
	</Tooltip.Root>
</Tooltip.Provider>
