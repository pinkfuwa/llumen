# llumen

繁體中文版，請參閱 [README.zh-tw.md](./README.zh-tw.md)。

> [!IMPORTANT]
> This is a fork form [llumen](https://github.com/pinkfuwa/llumen) for [hackathon](https://2025.meichuhackathon.org/)

llumen is a lightweight, self-hostable LLM chat application (frontend + backend) that aims to provide an out-of-the-box experience for self-hosting users.

## Highlights

- Good out of box experience
- Lighting fast and high quality UI
- Markdown rendering with code and math support.
- Feature multiple modes for different use cases (normal, web-search-enabled, agentic).

## Screenshots & Video

TODO: video link

![new chat](./screenshots/new-chat.png)
![chatroom](./screenshots/chatroom.png)
![setting](./screenshots/setting.png)

## Quickstart

> [!TIP]
> Use [reasoning-proxy](https://github.com/Eason0729/reasoning-proxy) to unlock advance feature with normal openai endpoint

### Docker (Recommended)

- The repository includes a multi-stage `Dockerfile` that builds the frontend and the backend and produces a small image that serves static files and runs the server.
- Example: build and run the container (binds port 80 by default).

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  docker pull ghcr.io/pinkfuwa/llumen:latest
```

### Without docker

> [!TIP]
> prebuild-binary is not up to dated

We Also provided prebuild binary in release, download it and extract.

## Environment variables

- `API_KEY` (required) — OpenRouter or equivalent provider API key.
- `DATABASE_URL` — database connection string. Default in Docker: `sqlite://data/db.sqlite?mode=rwc`.
- `BIND_ADDR` — address the backend binds to (default in Docker: `0.0.0.0:80`).

## Where to look for more documentation

- Development and build-from-source steps, advanced type generation and other developer-focused docs were moved to `DEV.md`. If you want to build locally or contribute code, read `DEV.md` first.
- Backend source: `backend/` (Rust).
- Frontend source: `frontend/` (SvelteKit).
- Migrations: `backend/migration/`.
