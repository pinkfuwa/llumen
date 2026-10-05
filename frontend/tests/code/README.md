# Code rendering browser checks

From `frontend`, run `pnpm test:browser` with Node.js 22.18 or newer and Chromium installed. Set `LLUMEN_TEST_BROWSER` to a Chromium executable when automatic discovery cannot find it. The runner checks the Playwright Chromium cache before trying `chromium-browser`; it does not install a browser or require Playwright.

The runner starts a local Vite server, launches a temporary headless browser profile, and exercises the application's code renderer, streaming Markdown parser, TOML editor, reasoning component, and chat viewport. It checks source preservation during streaming and replacement, highlighting without DOM mutation or geometry changes, six CSS theme variants without retokenization, editor alignment and selection, downward reasoning expansion in short and long histories, and plain-text fallback without the CSS Highlight API. The runner closes the server and browser after completion.

Streaming regressions pause the syntax worker and verify that the first snapshot starts immediately, appended source remains visible while existing colored ranges survive every sampled frame, updates coalesce to the latest snapshot after 100 ms, and fence closure retains colors while requesting final highlighting. The same checks exercise component identity through the application's incremental Markdown parser.

Run `pnpm test --run` for worker queue, token-offset, and range-cleanup tests. Browser geometry checks complement those tests because jsdom does not calculate layout.
