import { addMessages, init as initLocale, locale, waitLocale } from 'svelte-i18n';

import en from '../messages/en.json';
import zhTW from '../messages/zh-tw.json';
import zhCN from '../messages/zh-cn.json';

addMessages('en', en);
addMessages('zh-tw', zhTW);
addMessages('zh-cn', zhCN);

initLocale({
	fallbackLocale: 'en'
});

export function setLocale(language: 'en' | 'zh-tw' | 'zh-cn') {
	locale.set(language);
	waitLocale();
}
