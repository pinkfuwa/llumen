<script lang="ts">
	import Header from './Header.svelte';
	import Pagination from './Pagination.svelte';
	import Setting from '../setting/Setting.svelte';
	import { m } from '$lib/paraglide/messages';
	import OpenBtn from './OpenBtn.svelte';
	import { open } from './open.svelte';

	const sidebarStyle =
		'flex h-screen w-screen flex-col justify-between border-sidebar-border bg-card p-5 transition-all data-[state=close]:ml-[-100vw] md:w-[min(calc(160px+20rem),33vw)] md:border-r md:data-[state=close]:-ml-[min(calc(160px+20rem),33vw)]';
</script>

<OpenBtn />
<header class={sidebarStyle} data-state={open.val ? 'open' : 'close'}>
	<Header />
	<Pagination />
	<div class="mt-4 shrink-0 border-t border-border pt-4">
		<Setting />
	</div>
</header>
