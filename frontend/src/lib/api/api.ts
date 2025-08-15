import { dev } from '$app/environment';
import type { Error as APIError } from './types';

export const apiBase = dev ? 'http://localhost:8001/' : '/api/';

export async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}
