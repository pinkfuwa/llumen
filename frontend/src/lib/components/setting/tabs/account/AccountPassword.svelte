<script lang="ts">
	import { _ } from 'svelte-i18n';
	import CheckPwd from '../../CheckPwd.svelte';
	import { UpdateUser } from '$lib/api/user';
	import { clearCache } from '$lib/api/state';
	import { token } from '$lib/store';
	import Warning from '../../Warning.svelte';
	import { goto } from '$app/navigation';

	let { mutate, isError } = UpdateUser();
</script>

{#if $isError}
	<Warning>{ $_('setting.account.error_updating_password') }</Warning>
{/if}
<CheckPwd
	message={$_('setting.account.enter_new_password')}
	onsubmit={(password) => {
		mutate({ password }, () => {
			token.set(undefined);
			clearCache();
			goto('/login');
		});
	}}
></CheckPwd>
