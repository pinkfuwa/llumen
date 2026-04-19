## Project Overview

Privacy-focused LLM chat web-application optimized for minimal hardware (Raspberry Pi, old laptops).

> You probably should read BUILD.md for how to build llumen

## Architecture

llumen follows mono repo file structure:
- `./frontend`: Svelte 5 (static-adapter) frontend
- `./backend`: Axum backend
- `./agent`: Prompt and regression prompt test
- `./package`: Build script and deployment declaration
- `./docs`: Documentation with Docusaurus

## Tech Stack

- Rust / Axum / SeaORM / Tokio
- mLua with luau sandboxing
- typeshare (codegen)
- Svelte 5 (SPA) / TailwindCSS / Vite / TypeScript
- bit-ui
- shiki / lezer
- Custom markdown parser

## General Coding Guidelines

### Strong Size Awareness

Our project emphasizes performance and binary-size. We choose minimal dependency for both frontend and backend.

### Use Correct Tools

- Always use pnpm instead of npm
- Always use cargo xtask for scripting system (called by developer)
- Always use plain bash script in `packages` for build script

### Conventional Commits

The Conventional Commits specification is a lightweight convention on top of commit messages:
```
<type>: <description>

[optional body]

[optional footer(s)]
[BREAKING CHANGE: `function_name` changed]
```

### Don't Change Git Config

Please never change git config by running `git config --global user.name` or `git config --global user.email`

### Separation of Concerns

- Place UI and presentation logic in `frontend/src/lib/ui` (components, view helpers, styles). Keep this folder as the single place for UI-specific utilities and components.
- Keep route handlers in `backend/src/routes` and limit them to high-level request/response orchestration and input validation; routes should be thin.
- Put business/domain logic and data access into `backend/src/lib` (services, use-cases, repositories) so routes remain thin and reusable.
- Avoid mixing UI, business, and persistence code across layers; prefer clear module boundaries and single-responsibility functions.

### Always Check Before Commit

- If backend changed, run `cargo check`, `cargo test` and `cargo +nightly fmt`.
- If frontend changed, run `pnpm check` and `pnpm test`.
- If very complex logic changed, check `.github/workflows/check.yml` and try pass all CI tests.
- If dependency changed, regenerate third-party licenses by `pnpm run generate-licenses` in frontend and `cargo xtask gen-license` in backend.
- `pnpm build`, `cargo build` and `cargo test --release` is usually unnecessary.
- DO NOT run `pnpm dev` or `cargo xtask run`, those are intended for user to run!
