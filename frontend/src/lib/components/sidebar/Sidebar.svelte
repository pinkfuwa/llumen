<script lang="ts">
	import Header from './Header.svelte';
	import Pagination from './Pagination.svelte';
	import Setting from '../setting/Setting.svelte';
	import OpenBtn from './OpenBtn.svelte';
	import { open } from './open.svelte';
</script>

<OpenBtn />
<header
	class="flex h-screen w-screen flex-col justify-between border-sidebar-border bg-card p-5 transition-[margin] data-close:ml-[-100vw] md:w-[min(calc(160px+20rem),33vw)] md:border-r md:data-close:-ml-[min(calc(160px+20rem),33vw)]"
	data-close={open.val ? undefined : ''}
>
	<Header />
	<Pagination />
	<div class="mt-4 shrink-0 border-t border-border pt-4">
		<Setting />
	</div>
</header>
