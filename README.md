# llumen

> [!WARNING]
> The project is WIP, some feature are missing, see [roadmap](#roadmap).

llumen is a lightweight, self-hostable LLM chat application (frontend + backend) that aims to provide an out-of-the-box experience for self-hosting users.

Its design goal is simplicity: you only need a single OpenRouter API key to use LLM features — no separate keys for OCR, embeddings, image generation, or other services.

## Highlights

- Single API key requirement (OpenRouter) for model calls.
- Markdown rendering with code and math support.
- Multiple chat modes (normal, web-search-enabled).
- Roadmap: deep-research and agentic modes (WIP).

## Screenshots

![new chat](./screenshots/new-chat.png)
![chatroom](./screenshots/chatroom.png)
![setting](./screenshots/dark-setting.png)

## Quickstart: Docker

- The repository includes a multi-stage `Dockerfile` that builds the frontend and the backend and produces a small image that serves static files and runs the server.
- Example: build and run the container (binds port 80 by default).

```
docker build -t llumen .
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  llumen
```

## Environment variables

- `API_KEY` (required) — OpenRouter or equivalent provider API key.
- `DATABASE_URL` — database connection string. Default in Docker: `sqlite://data/db.sqlite?mode=rwc`.
- `BIND_ADDR` — address the backend binds to (default in Docker: `0.0.0.0:80`).

## Where to look for more documentation

- Development and build-from-source steps, advanced type generation and other developer-focused docs were moved to `DEV.md`. If you want to build locally or contribute code, read `DEV.md` first.
- Backend source: `backend/` (Rust).
- Frontend source: `frontend/` (SvelteKit).
- Migrations: `backend/migration/`.

## Roadmap

Ordered by priority

1. Customized system prompt
2. Title Generation
3. T support
4. File upload
5. Deep research mode
