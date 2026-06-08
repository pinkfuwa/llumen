import { addMessages, init, locale } from 'svelte-intl-precompile';
import { preference } from '$lib/preference/index.svelte';
import en from '$locales/en';
import zhCn from '$locales/zh-cn';
import zhTw from '$locales/zh-tw';

addMessages('en', en);
addMessages('zh-cn', zhCn);
addMessages('zh-tw', zhTw);

init({
	fallbackLocale: 'en',
	initialLocale: preference.value.locale
});

$effect.root(() => {
	$effect(() => {
		locale.set(preference.value.locale);
	});
	return () => {};
});
