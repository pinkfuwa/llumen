# Installation Guide

This guide covers all the ways to install and deploy Llumen.

## Docker (Recommended)

Docker is the recommended way to run Llumen as it includes all dependencies and is easy to update.

### Basic Docker Run

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:nightly
```

### Docker Compose

For persistent deployments, use Docker Compose:

```yaml
services:
  llumen:
    image: ghcr.io/pinkfuwa/llumen:nightly
    restart: on-failure:4
    environment:
      - "API_KEY=<YOUR_API_KEY>"
    volumes:
      - "./data:/data"
    ports:
      - "80:80"
    deploy:
      resources:
        limits:
          memory: 512M
```

Save this as `docker-compose.yml` and run:

```bash
docker compose up -d
```

### Additional Docker Compose Examples

More deployment examples are available in the repository:
- [OpenRouter setup](https://github.com/pinkfuwa/llumen/tree/main/docs/sample/openrouter.yaml)
- [Custom OpenAI endpoint](https://github.com/pinkfuwa/llumen/tree/main/docs/sample/openai.yaml)
- [Traefik reverse proxy](https://github.com/pinkfuwa/llumen/tree/main/docs/sample/traefik-proxy.yaml)

## Prebuilt Binaries

Prebuilt binaries are available for Linux and Windows.

> **Note**: Prebuilt binaries are only updated on releases, not nightly builds.

### Download

1. Go to the [releases page](https://github.com/pinkfuwa/llumen/releases)
2. Download the appropriate archive for your system:
   - `llumen-linux-x86_64.tar.gz` - Linux x86_64
   - `llumen-linux-arm64.tar.gz` - Linux ARM64
   - `llumen-windows-x86_64.zip` - Windows x86_64

### Installation

**Linux:**
```bash
tar -xzf llumen-linux-x86_64.tar.gz
chmod +x llumen
./llumen
```

**Windows:**
Extract the ZIP file and run `llumen.exe`.

## Building from Source

For development or custom builds, you can compile from source.

### Prerequisites

- Rust 1.89 or later
- Node.js 21+ with pnpm
- SQLite development libraries

### Build Steps

1. Clone the repository:
```bash
git clone https://github.com/pinkfuwa/llumen.git
cd llumen
```

2. Build the frontend:
```bash
cd frontend
pnpm install
pnpm run build
cd ..
```

3. Build the backend:
```bash
cd backend
cargo build --release
```

4. Run:
```bash
./target/release/backend
```

### Nix Users

A Nix flake is provided for development:

```bash
nix develop
```

## Production Considerations

### Data Persistence

Always mount the `/data` volume in Docker to persist:
- SQLite database (`db.sqlite`)
- Blob storage (`blobs.redb`)

### Reverse Proxy

When running behind a reverse proxy (nginx, Traefik, Caddy):

1. Ensure WebSocket support is enabled for SSE streaming
2. Set appropriate timeouts for long-running completions
3. Forward the `Authorization` header

Example nginx configuration:
```nginx
location / {
    proxy_pass http://localhost:80;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_read_timeout 300s;
}
```

## Updating

### Docker

```bash
docker pull ghcr.io/pinkfuwa/llumen:nightly
docker compose down
docker compose up -d
```

### Binary

Download the new release and replace the existing binary. Your data in the `data/` directory will be preserved.

## Next Steps

- Configure your instance: [Configuration](./configuration.md)
- Learn about features: [Features](./features.md)
