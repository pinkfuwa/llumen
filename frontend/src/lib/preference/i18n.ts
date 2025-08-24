import { addMessages, init as initLocale, locale, waitLocale } from 'svelte-i18n';

import en from '../messages/en.json';
import zhTW from '../messages/zh-tw.json';

addMessages('en', en);
addMessages('zh-tw', zhTW);

initLocale({
	fallbackLocale: 'en'
});

export function setLocale(language: 'en' | 'zh-tw') {
	locale.set(language);
	waitLocale();
}
