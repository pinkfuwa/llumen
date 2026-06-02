import { addMessages, init as initLocale, locale, waitLocale } from 'svelte-i18n';

import en from '../i18n/en.json';
import zhTW from '../i18n/zh-tw.json';
import zhCN from '../i18n/zh-cn.json';

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
