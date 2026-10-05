<script lang="ts">
	import '../../src/app.css';
	import Code from '$lib/ui/code/Code.svelte';
	import TomlEditor from '$lib/components/editor/TomlEditor.svelte';
	import Reasoning from '$lib/components/message/Reasoning.svelte';
	import ChatViewport from '$lib/ui/ChatViewport.svelte';
	import Markdown from '$lib/components/markdown/Root.svelte';
	import { addMessages, init } from 'svelte-intl-precompile';
	import { tick } from 'svelte';

	addMessages('en', { chat: { reasoning: 'Reasoning' } });
	init({ fallbackLocale: 'en', initialLocale: 'en' });

	let text = $state('// Initial source');
	let lang = $state('js');
	let incremental = $state(true);
	let value = $state('[server]\nport = 8001\nname = "你好 😀"\n\n');
	let longHistory = $state(false);
	let reasoning = $state(false);
	let markdownSource = $state<string | null>(null);

	export async function update(source: string, language = 'js', streaming = true) {
		markdownSource = null;
		text = source;
		lang = language;
		incremental = streaming;
		await tick();
	}

	export async function markdown(source: string) {
		markdownSource = source;
		await tick();
	}

	export async function history(long: boolean) {
		longHistory = long;
		reasoning = false;
		await tick();
	}
</script>

<div class="fixture" data-theme="llumen" data-dark="false">
	<div id="viewport-container" class="h-full">
		<ChatViewport>
			{#if longHistory}<div class="shrink-0" style="height:900px">Earlier history</div>{/if}
			<div id="before">Before reasoning</div>
			<Reasoning content={'Reasoning line\n'.repeat(12)} bind:open={reasoning} />
			<div id="code">
				{#if markdownSource !== null}
					<Markdown source={markdownSource} incremental />
				{:else}
					<Code {text} {lang} {incremental} />
				{/if}
			</div>
			<div id="after">After code</div>
			{#snippet composer()}Composer{/snippet}
		</ChatViewport>
	</div>
	<div id="editor"><TomlEditor bind:value /></div>
</div>

<style>
	.fixture {
		height: 450px;
		width: 640px;
		font-size: 16px;
	}
</style>
