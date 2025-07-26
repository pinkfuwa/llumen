<script lang="ts">
	import { Plus, Trash2 } from '@lucide/svelte';
	import ChatroomBtn from './buttons/ChatroomBtn.svelte';
</script>

<ul class="nobar max-h-[calc(100vh-185px)] overflow-y-scroll text-sm">
	<li>
		<a
			href="/chat/new"
			class="mb-2 flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold hover:bg-hover"
		>
			<Plus class="mr-2 h-5 w-5" />
			New
		</a>
	</li>
	<ChatroomBtn />
</ul>
