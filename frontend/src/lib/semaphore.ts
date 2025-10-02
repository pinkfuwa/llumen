// single-permit-semaphore.ts
export default class Semaphore {
	private waiters: Array<() => void> = [];

	private locked = false;

	/**
	 * Acquire the semaphore.
	 * The returned promise resolves when the caller obtains the permit.
	 *
	 * Example:
	 *   await semaphore.acquire();
	 *   // critical section
	 *   semaphore.release();
	 */
	async acquire(): Promise<void> {
		if (!this.locked) {
			// Permit is free – take it immediately.
			this.locked = true;
			return;
		}

		// Otherwise, wait for a release.
		return new Promise<void>((resolve) => {
			this.waiters.push(resolve);
		});
	}

	/**
	 * Release the semaphore, allowing the next waiter (if any) to proceed.
	 *
	 * Throws if called while the semaphore is not held.
	 */
	release(): void {
		if (!this.locked) {
			throw new Error('Cannot release an unlocked semaphore');
		}

		if (this.waiters.length > 0) {
			// Dequeue the next waiter and hand over the permit.
			const next = this.waiters.shift()!;
			// Keep `locked` true – the next holder now owns the permit.
			next();
		} else {
			// No waiters: simply mark the semaphore as free.
			this.locked = false;
		}
	}

	/**
	 * Convenience helper that runs an async callback inside the semaphore.
	 * Guarantees release even if the callback throws.
	 *
	 * @param fn The critical‑section function.
	 */
	async runExclusive<T>(fn: () => Promise<T>): Promise<T> {
		await this.acquire();
		try {
			return await fn();
		} finally {
			this.release();
		}
	}

	/** Check whether the permit is currently held. */
	isLocked(): boolean {
		return this.locked;
	}
}
