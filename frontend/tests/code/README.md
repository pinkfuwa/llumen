# Code rendering browser checks

From `frontend`, run `pnpm test:browser` with Node.js 22.18 or newer and Chromium installed. Set `LLUMEN_TEST_BROWSER` to a Chromium executable when automatic discovery cannot find it. The runner checks the Playwright Chromium cache before trying `chromium-browser`; it does not install a browser or require Playwright.

The runner starts a local Vite server, launches a temporary headless browser profile, and exercises the application's code renderer, TOML editor, reasoning component, and chat viewport. It checks source preservation during streaming and replacement, highlighting without DOM mutation or geometry changes, six CSS theme variants without retokenization, editor alignment and selection, downward reasoning expansion in short and long histories, and plain-text fallback without the CSS Highlight API. The runner closes the server and browser after completion.

Run `pnpm test --run` for worker queue, token-offset, and range-cleanup tests. Browser geometry checks complement those tests because jsdom does not calculate layout.
