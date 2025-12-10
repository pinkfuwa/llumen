<div align="center">
  <img src="frontend/static/web-app-manifest-512x512.png" alt="llumen Logo" width="200" height="auto" />

  # Llumen

  ### :rocket: The antidote to bloated AI interfaces.
  ### A lightweight, performant chat application for the rest of us.

  [![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)
  [![Nightly Docker](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml)
  [![status check](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml)
  ![MSRV](https://img.shields.io/static/v1?label=MSRV&message=1.89&color=orange&logo=rust)

  <img src="https://skillicons.dev/icons?i=rust,ts,svelte,tailwind" alt="Technology Stack" width="auto" height="70" />
</div>

---

## :bulb: Why we built llumen

### The Problem: The "Self-Hosted" Tradeoff: Powerful but Complex

If you have ever tried to self-host an LLM interface on a modest device, you know the struggle:
1.  **The Bloat:** Python-based containers that eat Gigabytes of RAM just to idle.
2.  **The Lag:** Waiting 30+ seconds for a server to boot and another minute to load chat history.
3.  **The Config Hell:** Spending hours wrestling with pipelines just to get a simple feature like "Title Generation" to work reliably.

### The Solution: Simplicity by Engineering

We refused to accept that "powerful" means "heavy." We built llumen to fill the gap between commercial products (easy to setup, but no privacy) and power-user tools (private, but heavy & complex).

| Feature | Typical "Power User" UI | **llumen** |
| :--- | :--- | :--- |
| **Asset Footprint** | HUGE (GBs) | **Tiny** (12MB) |
| **RAM Usage** | High (Nightmare to Debug) | **< 128MB** |
| **Setup Time** | Hours of config | **Zero-Config** |

## :sparkles: Features

Don't let the size fool you. Llumen is lightweight in resources, but heavy on capability.

- :electric_plug: **OpenAI Compatible:** Works with OpenRouter, local models, or OpenAI-compatible server.
- :rocket: **Blazing Fast:** Sub-second cold starts. No more waiting.
- :brain: **Smart & Deep:** Built-in "Deep Research" capabilities, web-search integration.
- :art: **Rich Media:** Handles PDF uploads, image generation, and renders complex LaTeX/Code.
- :handshake: **Run Anywhere:** Windows, Linux, Docker, and fully optimized for **Arm64** (yes, it fly on a Raspberry Pi).

[![Video preview](./docs/video.webp)](https://github.com/user-attachments/assets/4d46e649-bd33-4850-af2b-59527cc11618)

## :zap: Quickstart (The Proof)

Prove the speed yourself. If you have Docker, you are 30 seconds away from chatting.

> [!IMPORTANT]
> **Default Credentials:**
> - User: `admin`
> - Pass: `P@88w0rd`

### :whale: Docker (Recommended)

Our multi-stage build produces a tiny, efficient container.

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:latest
```

*That's it.* No pipelines to configure. No dependencies to install.

See [./docs/sample](./docs/sample) for docker-compose examples.

### :package: Other Methods
Prefer a binary? We support that too. Check the [Releases](https://github.com/pinkfuwa/llumen/releases) for Windows and Linux binaries.

## :key: Configuration (Optional)

It works out of the box, but if you want to tweak it:

- `API_KEY` (required) — Your OpenRouter/Provider key.
- `OPENAI_API_BASE` — Custom endpoint (Default: `https://openrouter.ai/api`).
- `DATABASE_URL` — SQLite path (Default: `sqlite://data/db.sqlite?mode=rwc`).
- `BIND_ADDR` — Network interface (Default: `0.0.0.0:80`).

## :book: Documentation

- **User Guide**: [./docs/user/README.md](./docs/user/README.md) - Full features and usage.
- **For Developers**:
  - Build from source: `./docs/chore/README.md`
  - Architecture docs: `./docs/dev/README.md`

<div align="center">
  Built with ❤️ by pinkfuwa. Keep it simple, keep it fast.
</div>
