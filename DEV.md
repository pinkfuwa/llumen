This document is aimed at contributors and maintainers who want to develop, build, and test llumen locally or produce production artifacts. It collects the development-oriented instructions that were moved out of `README.md`.

## Overview

llumen is a two-part application:

- `frontend/` — SvelteKit based SPA that provides the chat UI.
- `backend/` — Rust (axum + sea-orm) API server which also serves the built frontend static files in production images.
- `backend/migration` — migration CLI to manage the database schema.
- `prompts/` — built-in prompt templates.

Primary development flows:
- Fast local frontend development (hot reload).
- Local backend development (run using `cargo run`).
- Coordinated development (run frontend and backend separately).
- Produce production Docker image (recommended) or build static backend binary.

## Prerequisites

- Node.js 22+ (tested)
- pnpm (recommended) or npm
- Rust 1.89+ and Cargo
- (Optional) Docker/Docker Compose for containerized builds
- (Optional for static Linux binary) musl toolchain and system packages: `musl-tools`, `pkg-config`, `make`

## Environment variables used in development
- `API_KEY` — required for LLM provider (OpenRouter by default).
- `DATABASE_URL` — e.g. `sqlite://data/db.sqlite?mode=rwc`. Default used in Docker is `sqlite://data/db.sqlite?mode=rwc`.
- `BIND_ADDR` — address the backend binds to (default 0.0.0.0:80 in Docker).
- `STATIC_DIR` — path to static frontend files (default `/static` in Docker).

## Release: docker


```
docker build -t llumen .
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  llumen
```

## Scripts

TODO: document `backend/justfile`
