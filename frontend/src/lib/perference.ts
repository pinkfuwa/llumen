interface Preference {
	theme: 'light' | 'dark';
	language: 'en' | 'zh-tw';
}

export const defaultPreference: Preference = {
	theme: 'light',
	language: 'en'
};

function getCurrentPreference(): Preference {
	try {
		const storedPreference = localStorage.storable as string;
		if (storedPreference) {
			return JSON.parse(storedPreference) as Preference;
		}
	} catch (error) {
		console.warn('corrupted preference');
	}
	return defaultPreference;
}
