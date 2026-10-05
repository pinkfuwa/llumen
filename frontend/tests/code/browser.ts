import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readdir, rm } from 'node:fs/promises';
import { homedir, tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { createServer } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

const cache = path.join(homedir(), '.cache/ms-playwright');
const installed = await readdir(cache).catch(() => [] as string[]);
const chromium = installed.find((name) => /^chromium-\d+$/.test(name));
const executable =
	process.env.LLUMEN_TEST_BROWSER ??
	(chromium ? path.join(cache, chromium, 'chrome-linux64/chrome') : 'chromium-browser');
const profile = await mkdtemp(path.join(tmpdir(), 'llumen-code-browser-'));
const server = await createServer({
	configFile: false,
	plugins: [svelte(), tailwindcss()],
	optimizeDeps: {
		include: ['@speed-highlight/core/languages', '@speed-highlight/core/tokenize']
	},
	resolve: {
		alias: {
			$lib: path.resolve('src/lib'),
			'$app/state': path.resolve('tests/code/kit.ts'),
			'$app/navigation': path.resolve('tests/code/kit.ts'),
			'$app/environment': path.resolve('tests/code/kit.ts')
		}
	},
	server: { host: '127.0.0.1', port: 0 }
});
await server.listen();
const address = server.httpServer!.address();
assert(address && typeof address !== 'string');
const browser = spawn(
	executable,
	[
		'--headless',
		'--no-sandbox',
		'--disable-gpu',
		'--remote-debugging-port=0',
		`--user-data-dir=${profile}`,
		'about:blank'
	],
	{ stdio: ['ignore', 'ignore', 'pipe'] }
);
let socket: WebSocket | undefined;

try {
	const endpoint = await new Promise<string>((resolve, reject) => {
		let output = '';
		const timeout = setTimeout(
			() => reject(new Error(`Browser startup timed out: ${output}`)),
			10000
		);
		browser.on('error', (error) => {
			clearTimeout(timeout);
			reject(error);
		});
		browser.on('exit', (code) => {
			clearTimeout(timeout);
			reject(new Error(`Browser exited ${code}: ${output}`));
		});
		browser.stderr.on('data', (chunk: Buffer) => {
			output += chunk.toString();
			const match = output.match(/DevTools listening on (\S+)/);
			if (match) {
				clearTimeout(timeout);
				resolve(match[1]);
			}
		});
	});
	const targetUrl = new URL('/json', endpoint.replace('ws:', 'http:'));
	const targets = (await (await fetch(targetUrl)).json()) as {
		type: string;
		webSocketDebuggerUrl: string;
	}[];
	socket = new WebSocket(targets.find((target) => target.type === 'page')!.webSocketDebuggerUrl);
	await new Promise<void>((resolve, reject) => {
		socket!.onopen = () => resolve();
		socket!.onerror = reject;
	});
	let nextId = 0;
	const pending = new Map<
		number,
		{ resolve: (result: unknown) => void; reject: (error: Error) => void }
	>();
	socket.onmessage = (event) => {
		const message = JSON.parse(String(event.data)) as {
			id: number;
			result: unknown;
			error?: { message: string };
		};
		const request = pending.get(message.id);
		if (!request) return;
		pending.delete(message.id);
		if (message.error) request.reject(new Error(message.error.message));
		else request.resolve(message.result);
	};
	function call(method: string, params: object = {}) {
		return new Promise<unknown>((resolve, reject) => {
			const id = ++nextId;
			pending.set(id, { resolve, reject });
			socket!.send(JSON.stringify({ id, method, params }));
		});
	}
	async function evaluate<T>(expression: string): Promise<T> {
		const result = (await call('Runtime.evaluate', {
			expression,
			awaitPromise: true,
			returnByValue: true
		})) as {
			result: { value: T };
			exceptionDetails?: unknown;
		};
		if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
		return result.result.value;
	}
	await call('Page.enable');
	await call('Page.addScriptToEvaluateOnNewDocument', {
		source: `
		window.workerRequests = [];
		const postMessage = Worker.prototype.postMessage;
		Worker.prototype.postMessage = function(request) {
			window.workerRequests.push(request);
			return postMessage.call(this, request);
		};
	`
	});
	await call('Page.navigate', {
		url: `http://127.0.0.1:${address.port}/tests/code/index.html`
	});
	for (let attempts = 0; attempts < 100; attempts++) {
		if (
			await evaluate<boolean>('Boolean(window.codeFixture && document.querySelector("#code pre"))')
		)
			break;
		await delay(100);
	}
	assert(await evaluate<boolean>('Boolean(window.codeFixture)'), 'Fixture did not mount');
	await evaluate(`(() => {
		window.geometry = () => {
			const viewport = document.querySelector('#viewport-container > div');
			return {
				code: document.querySelector('#code').getBoundingClientRect().toJSON(),
				after: document.querySelector('#after').getBoundingClientRect().toJSON(),
				before: document.querySelector('#before').getBoundingClientRect().toJSON(),
				scrollTop: viewport.scrollTop, scrollHeight: viewport.scrollHeight,
				anchor: getComputedStyle(viewport).overflowAnchor
			};
		};
	})();`);

	const source = 'const greeting = "你好 café 😀";\r\n\t/* multiline\ncomment */\n\n';
	await evaluate(`window.codeFixture.update(${JSON.stringify(source)}, 'js', true)`);
	const geometry = await evaluate('window.geometry()');
	await evaluate(`(() => {
		window.firstCodeNode = document.querySelector('#code [data-code-offset]').firstChild;
		window.codeMutations = [];
		window.observer = new MutationObserver((records) => window.codeMutations.push(...records.map(record => record.type)));
		window.observer.observe(document.querySelector('#code'), { subtree:true, childList:true, characterData:true });
	})();`);
	for (let attempts = 0; attempts < 100; attempts++) {
		if (await evaluate('Boolean(CSS.highlights.get("syntax-kwd")?.size)')) break;
		await delay(100);
	}
	assert.deepEqual(
		await evaluate('window.geometry()'),
		geometry,
		'Syntax decoration changed geometry'
	);
	assert.equal(
		await evaluate('window.codeMutations.length'),
		0,
		'Syntax decoration changed source DOM'
	);
	assert(
		await evaluate(
			'window.firstCodeNode === document.querySelector("#code [data-code-offset]").firstChild'
		)
	);
	assert(
		await evaluate('Boolean(CSS.highlights.get("syntax-kwd")?.size)'),
		`Worker did not paint syntax: ${JSON.stringify(await evaluate('({ requests: window.workerRequests, highlights: Array.from(CSS.highlights.keys()) })'))}`
	);
	assert.equal(
		await evaluate(
			'Array.from(document.querySelectorAll("#code [data-code-offset]"), line => line.textContent).join("\\n")'
		),
		source
	);
	const requests = await evaluate<number>('window.workerRequests.length');
	const colors = new Set<string>();
	for (const theme of ['llumen', 'dracula', 'vitesse']) {
		for (const dark of ['false', 'true']) {
			await evaluate(
				`(() => { const fixture = document.querySelector('.fixture'); fixture.dataset.theme=${JSON.stringify(theme)}; fixture.dataset.dark=${JSON.stringify(dark)}; })()`
			);
			assert.deepEqual(await evaluate('window.geometry()'), geometry, 'Theme changed geometry');
			colors.add(
				await evaluate<string>(
					'getComputedStyle(document.querySelector("#code [data-code-offset]"), "::highlight(syntax-kwd)").color'
				)
			);
		}
	}
	assert.equal(
		await evaluate('window.workerRequests.length'),
		requests,
		'Theme retokenized source'
	);
	assert(colors.size > 1, 'CSS variables did not change painted token colors');
	console.log(
		'PASS: highlighting and six theme variants preserve source DOM, geometry, and scroll position'
	);

	await evaluate('window.observer.disconnect()');
	for (const replacement of [
		'const x = "unfinished',
		'<script>alert("text")</script>\n',
		'\t\n\n',
		source.repeat(2000)
	]) {
		await evaluate(`window.codeFixture.update(${JSON.stringify(replacement)}, 'js', true)`);
		assert.equal(
			await evaluate(
				'Array.from(document.querySelectorAll("#code [data-code-offset]"), line => line.textContent).join("\\n")'
			),
			replacement,
			'Streaming source lagged decoration'
		);
	}
	await evaluate(`window.codeFixture.update(${JSON.stringify(source)}, 'toml', false)`);
	await delay(500);
	assert.equal(
		await evaluate(
			'Array.from(document.querySelectorAll("#code [data-code-offset]"), line => line.textContent).join("\\n")'
		),
		source
	);
	console.log(
		'PASS: streaming, large blocks, replacement, language changes, and finalization preserve complete source'
	);

	const editorSource = '[server]\nname = "你好 😀"\nport = 8001\n\n';
	await evaluate(`(() => {
		const textarea = document.querySelector('#editor textarea');
		textarea.value = ${JSON.stringify(editorSource)};
		textarea.dispatchEvent(new InputEvent('input', {bubbles: true, inputType: 'insertText'}));
		textarea.focus();
		textarea.setSelectionRange(15, 17);
	})()`);
	await delay(150);
	assert.equal(await evaluate('document.querySelector("#editor textarea").value'), editorSource);
	assert.equal(
		await evaluate(
			'Array.from(document.querySelectorAll("#editor [data-code-offset]"), line => line.textContent).join("\\n")'
		),
		editorSource
	);
	assert.equal(await evaluate('document.querySelector("#editor textarea").selectionStart'), 15);
	assert.equal(await evaluate('document.querySelector("#editor textarea").selectionEnd'), 17);
	const editorMetrics = await evaluate<{
		textarea: string[];
		overlay: string[];
		top: number;
		codeTop: number;
	}>(`(() => {
		const textarea = document.querySelector('#editor textarea');
		const pre = document.querySelector('#editor pre');
		const metrics = (element) => ['fontFamily', 'fontSize', 'lineHeight', 'tabSize'].map(key => getComputedStyle(element)[key]);
		return { textarea: metrics(textarea), overlay: metrics(pre), top: textarea.getBoundingClientRect().top + 8, codeTop: pre.getBoundingClientRect().top };
	})()`);
	assert.deepEqual(
		editorMetrics.textarea,
		editorMetrics.overlay,
		'Editor overlay changed text metrics'
	);
	assert.equal(editorMetrics.top, editorMetrics.codeTop, 'Editor overlay misaligned with textarea');
	console.log('PASS: TOML overlay preserves source, selection, and textarea alignment');

	for (const long of [false, true]) {
		await evaluate(`window.codeFixture.history(${long})`);
		await evaluate('document.querySelector("#viewport-container > div").scrollTop = 60');
		const before = await evaluate<{
			before: { y: number };
			after: { y: number };
			scrollTop: number;
			anchor: string;
		}>('window.geometry()');
		await evaluate('document.querySelector("[data-collapsible-trigger]").click()');
		await delay(300);
		const after = await evaluate<{
			before: { y: number };
			after: { y: number };
			scrollTop: number;
			anchor: string;
		}>('window.geometry()');
		assert.equal(
			after.before.y,
			before.before.y,
			`Reasoning pulled earlier content upward (${long ? 'long' : 'short'} history)`
		);
		assert.equal(after.scrollTop, before.scrollTop, 'Reasoning changed scroll position');
		assert.equal(after.anchor, 'none');
		assert(after.after.y > before.after.y, 'Reasoning did not expand downward');
	}
	console.log(
		'PASS: reasoning expands downward in short and long histories with anchoring disabled'
	);

	await call('Page.addScriptToEvaluateOnNewDocument', {
		source: 'Object.defineProperty(CSS, "highlights", {value: undefined});'
	});
	await call('Page.reload');
	for (let attempts = 0; attempts < 100; attempts++) {
		if (await evaluate('Boolean(window.codeFixture)')) break;
		await delay(100);
	}
	await evaluate(`window.codeFixture.update(${JSON.stringify(source)}, 'js', false)`);
	assert.equal(
		await evaluate(
			'Array.from(document.querySelectorAll("#code [data-code-offset]"), line => line.textContent).join("\\n")'
		),
		source
	);
	assert.equal(
		await evaluate('window.workerRequests.length'),
		0,
		'Unsupported browser started syntax worker'
	);
	console.log(
		'PASS: browsers without the highlight API retain complete plain source without a worker'
	);
} finally {
	socket?.close();
	browser.kill();
	await server.close();
	await rm(profile, { recursive: true, force: true }).catch(() => {});
}
