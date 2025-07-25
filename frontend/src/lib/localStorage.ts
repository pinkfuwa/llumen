import { localState } from '@sv-use/core';
import { setLocale } from '$lib/paraglide/runtime';

export function theme(): { current: 'light' | 'dark' } {
	return localState('theme', 'light');
}

type languageType = 'en' | 'zh-tw';
export function language(): { current: languageType } {
	let language = localState('language', 'en');

	return {
		get current(): languageType {
			return language.current as languageType;
		},
		set current(value: languageType) {
			language.current = value;
			setLocale(value);
		}
	};
}

type tokenType = string | undefined;
const undefinedToken = 'undefined';
export function token(): { current: tokenType } {
	let token = localState('token', undefinedToken);

	return {
		get current(): tokenType {
			return token.current === undefinedToken ? undefined : token.current;
		},
		set current(value: tokenType) {
			token.current = value === undefined ? undefinedToken : value;
		}
	};
}
