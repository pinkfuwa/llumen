import { localState } from '@sv-use/core';

interface Current<T> {
	current: T;
}

export const token = () => localState('token', '');

export const language = () => localState('language', 'en') as Current<'en' | 'zh-tw'>;
export const theme = () => localState('theme', 'light') as Current<'light' | 'dark'>;
