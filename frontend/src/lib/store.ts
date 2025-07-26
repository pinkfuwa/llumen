import { localState } from '@sv-use/core';

interface Current<T> {
	current: T;
}

export const useToken = () => localState('token', null) as Current<string | null>;

export const useLanguage = () => localState('language', 'en') as Current<'en' | 'zh-tw'>;
export const useTheme = () => localState('theme', 'light') as Current<'light' | 'dark'>;
