<div align="center">
  <img src="frontend/static/web-app-manifest-512x512.png" alt="llumen Logo" width="200" height="auto" />

  # Llumen

  ### :zap: a lightweight, performant LLM chat application with effortless setup :rocket:

  [![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
  [![Nightly Docker](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml)
  [![Release Binaries](https://github.com/pinkfuwa/llumen/actions/workflows/build-binaries.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/build-binaries.yml)

  <img src="https://skillicons.dev/icons?i=rust,ts,svelte,tailwind" alt="llumen Logo" width="auto" height="70" />
</div>

llumen is a lightweight, self-hostable LLM chat application (frontend + backend) that aims to provide an out-of-the-box experience for self-hosting users.

## :question: Why choose llumen?

- :electric_plug: OpenAI Compatible: Works with any **OpenAI-compatible** endpoint (OpenRouter, local models, etc.)
- :ship: Zero-Config Setup: Pre-integrated with OpenRouter for instant deployment
- :handshake: Multi-Platform Distribution: Windows executables, Docker images, Linux binaries with Arm64 support
- :rocket: Blazing Fast: Sub-second cold start, **30× smaller asset footprint** and <128MB RAM usage
- :blue_book: Rich Markdown: Full code syntax highlighting and LaTeX math support
- :mag: Advanced Modes: Standard chat, web-search integration, and **deep-research** capabilities :brain:
- :computer: Rich Features: **Message editing**, file uploads (PDFs, images, code)

## :star2: Screenshots

![deep-research](./docs/screenshots/deep-research.png)

![search](./docs/screenshots/search.png)

## :point_right: Quickstart

> [!IMPORTANT]
> The default account is `admin`, password is `P@88w0rd`

### :whale: Docker (Recommended)

- The repository includes a multi-stage [Dockerfile](./package/Dockerfile) that builds the frontend and the backend and produces a small image that serves static files and runs the server.
- Example: run the container (binds port 80 by default).

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:nightly
```
```
09:25:46.117Z INFO | Using endpoint https://openrouter.ai/api/v1/chat/completions for completions
09:25:46.180Z INFO | Listening on http://127.0.0.1:8001
09:25:46.295Z INFO | 344 models available
```

More docker-compose deployment sample [here](./docs/sample) :point_left:

### :package: Without docker

> [!TIP]
> Prebuild-binary is not up to dated, it only built on per-release basis

We also provided prebuild binary, download it and extract from [release](https://github.com/pinkfuwa/llumen/releases).

## :key: Environment variables

- `API_KEY` (required) — OpenRouter or equivalent provider API key.
- `API_BASE` — openai compatible api url. Default: `https://openrouter.ai/api`
- `DATABASE_URL` — database connection string. Default in Docker: `sqlite://data/db.sqlite?mode=rwc`.
- `BLOB_URL` — path for [redb](https://www.redb.org/) object storage. Default in Docker: `/data/blobs.redb`.
- `BIND_ADDR` — address the backend binds to (default in Docker: `0.0.0.0:80`).

## :book: Where to look for more documentation

- Development and build-from-source steps and other developer-focused docs were moved to `./docs/overview.md`
- User guide at `./docs/user.md`
