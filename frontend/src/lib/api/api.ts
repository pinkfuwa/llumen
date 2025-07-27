import { dev } from '$app/environment';

export const apiBase = dev ? 'http://localhost:8001/' : '/api/';

export async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}
