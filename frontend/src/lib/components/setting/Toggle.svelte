<script lang="ts">
	import Option from './Option.svelte';

	let {
		trueLabel,
		falseLabel,
		value = $bindable(true),
		...props
	}: {
		title: string;
		value?: boolean;
		trueLabel: string;
		falseLabel: string;
		disabled?: boolean;
		onchange?: () => void;
	} = $props();

	let selected = $derived(Boolean(value).toString());
	let data = $derived([
		{ value: 'false', label: falseLabel },
		{ value: 'true', label: trueLabel }
	]);
</script>

<Option
	bind:selected
	{data}
	{...props}
	onchange={() => {
		value = selected == 'true';
	}}
></Option>
